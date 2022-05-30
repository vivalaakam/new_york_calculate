import json
from decimal import Decimal

from .get_id import get_result_id


def format_result(item, weight_id, result):
    return json.loads(json.dumps({
        'id': get_result_id(item['id'], weight_id),
        'applicant_id': item['id'],
        'weight_id': weight_id,
        'score': result['wallet'],
        'results': result,
        'version': 1,
    }), parse_float=Decimal)
