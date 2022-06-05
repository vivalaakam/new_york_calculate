import datetime


def debug_best(tag, item, best, epoch):
    print("{}: {:>8} {} {:>3} {:>} {:>8.4f} {}".format(datetime.datetime.now(), tag, item['id'], epoch, best['epoch'],
                                                       best['score'],
                                                       best['weight_id'] if 'weight_id' in best else None))
