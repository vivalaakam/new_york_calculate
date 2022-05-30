import hashlib

from .random_id import random_id


def get_applicant_id(interval, start, end):
    return hashlib.md5("{}:{}:{}".format(interval, start, end).encode()).hexdigest()


def get_weight_id(applicant_id):
    return hashlib.md5("{}:{}".format(applicant_id, random_id(10)).encode()).hexdigest()


def get_result_id(applicant_id, weight_id):
    return hashlib.md5("{}:{}".format(applicant_id, weight_id).encode()).hexdigest()
