import json
from shlex import quote
from urllib.parse import urlencode

import requests


def prepare_params(params):
    for k, v in params.items():
        params[k] = json.dumps(v)

    return urlencode(params)


def create_batch(model, data):
    return {
        "method": "POST",
        "path": "/parse/classes/{}".format(model),
        "body": data
    }


def update_batch(model, object_id, data):
    return {
        "method": "PUT",
        "path": "/parse/classes/{}/".format(model, object_id),
        "body": data
    }


class Parse:
    def __init__(self, remote_url, app_id, rest_key, master_key='', debug=False):
        self.headers = {
            "X-Parse-Application-Id": app_id,
            "X-Parse-REST-API-Key": rest_key,
            "Content-Type": "application/json",
            "Accept": "application/json, text/plain, */*"
        }

        if master_key != '':
            self.headers['X-Parse-Master-Key'] = master_key

        self.debug = debug
        self.remote = remote_url

    def get_request(self, remote, **kwargs):
        params = prepare_params(kwargs)
        req = requests.get("{}{}?{}".format(self.remote, remote, params), headers=self.headers)

        if self.debug:
            print(to_curl(req.request))

        return req.json()

    def delete_request(self, remote, **kwargs):
        params = prepare_params(kwargs)
        req = requests.delete("{}{}?{}".format(self.remote, remote, params), headers=self.headers)

        if self.debug:
            print(to_curl(req.request))

        return req.json()

    def post_request(self, remote, data=None, **kwargs):
        params = prepare_params(kwargs)

        if data is None:
            data = {}

        req = requests.post("{}{}?{}".format(self.remote, remote, params), data=json.dumps(data),
                            headers=self.headers)
        if self.debug:
            print(to_curl(req.request))

        return req.json()

    def put_request(self, remote, data=None, **kwargs):
        params = prepare_params(kwargs)

        if data is None:
            data = {}

        req = requests.put("{}{}?{}".format(self.remote, remote, params), data=json.dumps(data),
                           headers=self.headers)

        if self.debug:
            print(to_curl(req.request))

        return req.json()

    def get_applicant(self, applicant_id):
        return self.get_request("/classes/Applicants/{}".format(applicant_id))

    def get_model(self, model_id):
        return self.get_request("/classes/Models/{}".format(model_id))

    def get_model_weights(self, model_weight_id):
        return self.get_request("/classes/ModelWeights/{}".format(model_weight_id))

    def batch(self, request_items):
        return self.post_request("/batch", {
            "requests": request_items
        })


def to_curl(request, compressed=False, verify=True):
    """
    Returns string with curl command by provided request object
    Parameters
    ----------
    compressed : bool
        If `True` then `--compressed` argument will be added to result
    """
    parts = [
        ('curl', None),
        ('-X', request.method),
    ]

    for k, v in sorted(request.headers.items()):
        parts += [('-H', '{0}: {1}'.format(k, v))]

    if request.body:
        body = request.body
        if isinstance(body, bytes):
            body = body.decode('utf-8')
        parts += [('-d', body)]

    if compressed:
        parts += [('--compressed', None)]

    if not verify:
        parts += [('--insecure', None)]

    parts += [(None, request.url)]

    flat_parts = []
    for k, v in parts:
        if k:
            flat_parts.append(quote(k))
        if v:
            flat_parts.append(quote(v))

    return ' '.join(flat_parts)
