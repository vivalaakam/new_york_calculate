import json

import requests


class Parse:
    def __init__(self, remote_url, app_id, master_key):
        self.headers = {
            "X-Parse-Application-Id": app_id,
            "X-Parse-Master-Key": master_key,
            "Content-Type": "application/json",
            "Accept": "application/json, text/plain, */*"
        }

        self.remote = remote_url

    def get_request(self, remote, params=None):
        if params is None:
            params = {}

        req = requests.get("{}{}?{}".format(self.remote, remote, json.dumps(params)), headers=self.headers)
        return req.json()

    def delete_request(self, remote, params=None):
        if params is None:
            params = {}

        req = requests.delete("{}{}?{}".format(self.remote, remote, json.dumps(params)), headers=self.headers)
        return req.json()

    def post_request(self, remote, params=None, data=None):
        if params is None:
            params = {}

        if data is None:
            data = {}

        req = requests.post("{}{}?{}".format(self.remote, remote, json.dumps(params)), data=json.dumps(data),
                            headers=self.headers)
        return req.json()

    def put_request(self, remote, params=None, data=None):
        if params is None:
            params = {}

        if data is None:
            data = {}

        req = requests.put("{}{}?{}".format(self.remote, remote, json.dumps(params)), data=json.dumps(data),
                            headers=self.headers)
        return req.json()

    def get_applicant(self, applicant_id):
        return self.get_request("/classes/Applicants/{}".format(applicant_id))

    def get_model(self, model_id):
        return self.get_request("/classes/Models/{}".format(model_id))
