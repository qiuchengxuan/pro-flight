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


def read_field(cli: CLI, field: str) -> numpy.array:
    output = json.loads(cli.tx('telemetry'))
    axes = jsonpath(output, 'imu.%s' % field)[0]
    return numpy.array(axes)
