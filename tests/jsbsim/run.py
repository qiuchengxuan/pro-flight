#!/usr/bin/env python3

import argparse
import os
import platform
import random
import signal
import subprocess
import sys
import time

import pexpect
from lxml import etree
from lxml.builder import E, ElementMaker

from jsbsim import JSBSim
from jsbsim.fcs import Control
from simulator import GNSS, Axes
from simulator import Control as Input
from simulator import Fixed, Position, Simulator

XSI = 'http://www.w3.org/2001/XMLSchema-instance'
HREF = 'http://jsbsim.sf.net/JSBSimScript.xsl'
SIMULATE_TIME = 10.0  # seconds
ALTIMETER_RATE = 10
GNSS_RATE = 10

RASCAL_XML = 'aircraft/rascal/rascal.xml'


def initialize() -> str:
    initialize = E.initialize(
        E.ubody('50.0', unit='KTS'),
        E.vbody('0.0', unit='FT/SEC'),
        E.wbody('0.0', unit='FT/SEC'),
        E.longitude('-95.163839', unit='DEG'),
        E.latitude('29.593978', unit='DEG'),
        E.phi('0.0', unit='DEG'),
        E.theta('0.0', unit='DEG'),
        E.psi('0.0', unit='DEG'),
        E.altitude('100', unit='FT'),
        E.elevation('2000.0', unit='FT'),
        E.hwind('0.0'),
        name='takeoff'
    )
    return etree.tostring(initialize, encoding='utf-8', xml_declaration=True, pretty_print=True)


def make_script(input_port: int) -> str:
    events = [
        E.event(
            E.description('Start the engine'),
            E.condition('simulation/sim-time-sec le 0.01'),
            E.set(name='propulsion/engine[0]/set-running', value='1'),
            E.set(name='accelerations/Nz', value='1.0'),
            name='Set engine running'
        ),
    ]
    xml = ElementMaker(nsmap=dict(xsi=XSI))
    location = '{%s}noNamespaceSchemaLocation' % XSI
    run_script = xml.runscript(
        E.use(aircraft='rascal', initialize='takeoff'),
        E.input(port=str(input_port)),
        E.run(*events, start='0.0', end=str(SIMULATE_TIME), dt='0.0001'),
        name='Rascal take off',
        **{location: 'http://jsbsim.sf.net/JSBSimScript.xsd'}
    )
    run_script.addprevious(etree.PI('xml-stylesheet', 'type="text/xsl" href="%s"' % HREF))
    root = run_script.getroottree()
    return etree.tostring(root, encoding='utf-8', xml_declaration=True, pretty_print=True)


def start_jsbsim(port: int):
    cmd = '/usr/bin/env JSBSim --realtime --nice --root=. rascal_test.xml'
    jsbsim = pexpect.spawn(cmd)
    if platform.system() == 'Darwin':
        jsbsim.expect('JSBSim Execution beginning')
    else:
        jsbsim.expect('Creating input TCP socket on port')
        index = jsbsim.expect([
            'Successfully bound to TCP input socket on port', 'Could not bind to TCP input socket'
        ])
        if index >= 1:
            print('Bind socket failed')
            return
    return jsbsim


def start_simulator(simulator: str, sock: str, simulator_config: str):
    cmd = 'RUST_LOG=debug %s -l %s --config %s' % (simulator, sock, simulator_config)
    cmd += ' --rate 100 --altimeter-rate 10'
    return subprocess.Popen(cmd, shell=True)


def jsbsim_to_simulator(jsbsim_api: JSBSim, simulator_api: Simulator, time_ms: int):
    simulator_api.update_input(Input(throttle=1.0))
    altitude_cm = jsbsim_api.altitude * 30.48
    if time_ms % (1000 / ALTIMETER_RATE) == 0:
        simulator_api.update_altitude(int(altitude_cm))
    if time_ms % (1000 / GNSS_RATE) == 0:
        p = jsbsim_api.position
        v = jsbsim_api.velocity
        gnss = GNSS(
            fixed=Fixed(
                Position(str(p.latitude), str(p.longitude), int(altitude_cm)),
                round(v.course(), 1),
                jsbsim_api.speed.gs,
                [int(v.x * 303), int(v.y * 303), int(v.z * 303)],  # ft/s to mm/s
                round(jsbsim_api.attitude.true_heading, 1)
            )
        )
        simulator_api.update_gnss(gnss)
    accel = jsbsim_api.acceleration
    simulator_api.update_acceleration(Axes(accel.x, accel.y, accel.z))
    gyro = jsbsim_api.gyro
    simulator_api.update_gyro(Axes(gyro.x, gyro.y, gyro.z))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--simulator', help='Pro-flight simulator executable path', type=str)
    parser.add_argument('--simulator-config', help='Pro-flight config path', type=str)
    args = parser.parse_args()
    os.chdir(os.path.dirname(sys.argv[0]))
    if os.path.exists(RASCAL_XML):
        os.remove(RASCAL_XML)
    os.symlink('Rascal110-JSBSim.xml', RASCAL_XML)
    with open('aircraft/rascal/takeoff.xml', 'wb') as f:
        f.write(initialize())

    jsbsim_port = random.randint(1000, 10000)
    with open('rascal_test.xml', 'wb') as f:
        f.write(make_script(jsbsim_port))
    jsbsim = start_jsbsim(jsbsim_port)
    jsbsim_api = JSBSim(jsbsim_port)
    jsbsim_api.hold()

    sock = '/tmp/simulator.sock'
    simulator = start_simulator(args.simulator, sock, args.simulator_config)
    simulator_api = Simulator(sock)

    while not os.path.exists('/tmp/simulator.sock'):
        time.sleep(0.1)

    print('total %fs' % SIMULATE_TIME)
    begin = time.time()
    elapsed = 0.0
    while elapsed < SIMULATE_TIME:
        now = time.time()
        elapsed = now - begin
        try:
            time_ms = int(elapsed * 1000)
            jsbsim_to_simulator(jsbsim_api, simulator_api, time_ms)
            telemetry = simulator_api.get_telemetry()
            atti = jsbsim_api.attitude
            fcs = telemetry.fcs.control.normalize()
            jsbsim_api.control(Control(fcs.engines[0], fcs.aileron_right, fcs.elevator, fcs.rudder))
            status = '%dkt, %dft,' % (jsbsim_api.speed.cas, jsbsim_api.height)
            atti = jsbsim_api.attitude
            gyro = jsbsim_api.gyro
            rate = 'gyro={%.1f %.1f, %.1f}' % (gyro.y, gyro.x, gyro.z)
            attitude = 'atti={%.1f %.1f, %.1f}' % (atti.roll, atti.pitch, atti.true_heading)
            control = 'ctrl={T: %.2f| %.2f %.2f %.2f}' % (
                fcs.engines[0], fcs.aileron_right, fcs.elevator, fcs.rudder
            )
            print('%.3fs: %s' % (elapsed, ' '.join([status, attitude, rate, control])))
            simulator_api.tick()
            if jsbsim_api.height <= 1:
                print('Crashed after %.3fs' % elapsed)
                break
        except KeyboardInterrupt:
            break
    jsbsim.kill(signal.SIGINT)
    simulator.kill()
    for path in [RASCAL_XML, 'aircraft/rascal/takeoff.xml', 'rascal_test.xml', sock]:
        try:
            os.remove(path)
        except FileNotFoundError:
            pass


if __name__ == '__main__':
    main()
