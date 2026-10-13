def moving_average(values, k):
    """Average of every window of k consecutive values."""
    out = []
    for i in range(len(values) - k):  # window loop
        window = values[i:i + k]
        out.append(sum(window) / k)
    return out


def group_by_customer(orders, groups={}):
    for order in orders:
        groups.setdefault(order["customer"], []).append(order)
    return groups


def mean(values):
    return sum(values) / len(values)
