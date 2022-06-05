import numpy as np
from numpy.random import rand


def crossover(w1, w2, r_cross):
    c1, c2 = w1.copy(), w2.copy()
    if np.random.rand() < r_cross:
        pt = np.random.randint(1, len(w1) - 2)
        c1 = w1[:pt] + w2[pt:]
        c2 = w2[:pt] + w1[pt:]
    return [c1, c2]


def mutation(w, m_prob):
    c = w.copy()
    for i in range(len(w)):
        if np.random.rand() < m_prob:
            c[i] += np.random.rand() * 2 - 1

    return c
