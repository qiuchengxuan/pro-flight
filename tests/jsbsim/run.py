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
from jsbsim import JSBSim
from jsbsim.fcs import Control
from lxml import etree
from lxml.builder import E, ElementMaker
from simulator import Axes
from simulator import Control as Input
from simulator import Simulator

XSI = 'http://www.w3.org/2001/XMLSchema-instance'
HREF = 'http://jsbsim.sf.net/JSBSimScript.xsl'
SIMULATE_TIME = 0.2  # seconds
DELTA_TIME = 0.001  # seconds
ALTIMETER_RATE  = 10

RASCAL_XML = 'aircraft/rascal/rascal.xml'


def initialize() -> str:
    initialize = E.initialize(
        E.ubody('0.0', unit='FT/SEC'),
        E.vbody('0.0', unit='FT/SEC'),
        E.wbody('0.0', unit='FT/SEC'),
        E.longitude('-95.163839', unit='DEG'),
        E.latitude('29.593978', unit='DEG'),
        E.phi('0.0', unit='DEG'),
        E.theta('0.0', unit='DEG'),
        E.psi('0.0', unit='DEG'),
        E.altitude('6.6', unit='FT'),
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
            name='Set engine running'
        )
    ]
    xml = ElementMaker(nsmap=dict(xsi=XSI))
    location = '{%s}noNamespaceSchemaLocation' % XSI
    run_script = xml.runscript(
        E.use(aircraft='rascal', initialize='takeoff'),
        E.input(port=str(input_port)),
        E.run(*events, start='0.0', end=str(SIMULATE_TIME / DELTA_TIME), dt=str(DELTA_TIME)),
        name='Rascal take off',
        **{location: 'http://jsbsim.sf.net/JSBSimScript.xsd'}
    )
    run_script.addprevious(etree.PI('xml-stylesheet', 'type="text/xsl" href="%s"' % HREF))
    root = run_script.getroottree()
    return etree.tostring(root, encoding='utf-8', xml_declaration=True, pretty_print=True)


def start_jsbsim(port: int):
    cmd = '/usr/bin/env JSBSim --realtime --root=. rascal_test.xml'
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
    cmd += ' --rate 1000 --altimeter-rate 10'
    print(cmd)
    return subprocess.Popen(cmd, shell=True)


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

    sock = '/tmp/simulator.sock'
    simulator = start_simulator(args.simulator, sock, args.simulator_config)
    simulator_api = Simulator(sock)

    while not os.path.exists('/tmp/simulator.sock'):
        time.sleep(0.1)

    total = int(SIMULATE_TIME / DELTA_TIME)
    try:
        for i in range(total):
            simulator_api.update_input(Input(throttle=1.0))
            accel = jsbsim_api.acceleration
            simulator_api.update_acceleration(Axes(accel.x, accel.y, accel.z))
            gyro = jsbsim_api.gyro
            simulator_api.update_gyro(Axes(gyro.roll, gyro.pitch, gyro.yaw))
            if i % ALTIMETER_RATE == 0:
                simulator_api.update_altitude(jsbsim_api.altitude * 30.48)

            output = simulator_api.get_output()
            jsbsim_api.step(Control(output.engine, output.aileron, output.elevator, output.rudder))
            status = 'speed: %d, height: %d' % (jsbsim_api.speed.cas, jsbsim_api.height)
            print('Iteration %d/%d, ' % (i + 1, total) + status, end='\r')
            if jsbsim_api.height <= 1:
                print('\nCrashed')
                break
    except KeyboardInterrupt:
        pass
    finally:
        jsbsim.kill(signal.SIGINT)
        simulator.kill()
        for path in [RASCAL_XML, 'aircraft/rascal/takeoff.xml', 'rascal_test.xml', sock]:
            os.remove(path)


if __name__ == '__main__':
    main()
