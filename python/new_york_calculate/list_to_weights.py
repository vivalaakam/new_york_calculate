import numpy as np


def list_to_weights(layers, list):
    prev = 0
    ret = []
    for layer in layers:
        curr = list[prev:prev + layer['size']]

        ret.append(np.reshape(curr, layer['shape']))

        prev += layer['size']

    return ret
