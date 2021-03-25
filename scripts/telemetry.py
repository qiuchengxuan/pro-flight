#!/usr/bin/python3

import json

import numpy
import yaml
from cli import CLI
from jsonpath import jsonpath


def get_bias(cli: CLI, sensor: str) -> numpy.array:
    output = yaml.load(cli.tx('show'), Loader=yaml.FullLoader)
    axes = jsonpath(output, 'imu.%s.bias' % sensor)[0]
    return numpy.array([axes['x'], axes['y'], axes['z']])


def get_sensitive(cli: CLI, sensor: str) -> int:
    output = json.loads(cli.tx('telemetry'))
    return jsonpath(output, 'sensor.%s.sensitive' % sensor)[0]


def read_sensor(cli: CLI, sensor: str) -> numpy.array:
    output = json.loads(cli.tx('telemetry'))
    axes = jsonpath(output, 'sensor.%s.axes' % sensor)[0]
    return numpy.array([axes['x'], axes['y'], axes['z']])
