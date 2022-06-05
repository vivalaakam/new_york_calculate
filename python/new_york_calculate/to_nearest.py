import math


def floor_to_nearest(num, base):
    return math.floor(num / base) * base


def ceil_to_nearest(num, base):
    return math.ceil(num / base) * base
