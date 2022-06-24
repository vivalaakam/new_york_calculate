import datetime


def debug_best(tag, item, best, epoch):
    print("{}: {:>8} {} {:>3} {:>} {:>8.4f} {}".format(datetime.datetime.now(), tag, item['objectId'], epoch,
                                                       best['epoch'], best['score'],
                                                       best['weightId'] if 'weightId' in best else None))
