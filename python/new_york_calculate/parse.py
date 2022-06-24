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

    def get_applicant(self, applicant_id):
        item_json = requests.get("{}/classes/Applicants/{}".format(self.remote, applicant_id), headers=self.headers)
        return item_json.json()

    def get_model(self, model_id):
        model_json = requests.get("{}/classes/Models/{}".format(self.remote, model_id), headers=self.headers)
        return model_json.json()
