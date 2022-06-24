import json

from new_york_calculate import Parse


def test_parse():
    parse = Parse("https://httpbin.org", app_id="", rest_key="")

    resp = parse.get_request("/get", where={"a": "b"}, limit=10)
    assert json.dumps(resp['args']) == '{"limit": "10", "where": "{\'a\': \'b\'}"}'

    resp = parse.get_request("/get", where={"a": {"$in": ["b"]}}, limit=10)
    assert json.dumps(resp['args']) == '{"limit": "10", "where": "{\'a\': {\'$in\': [\'b\']}}"}'
