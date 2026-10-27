"""Summaries and budget marks for parity performance results."""
import gzip
import math
import statistics
import tomllib
from pathlib import Path

BUDGETS = Path(__file__).resolve().parent / 'budgets.toml'
# Result field -> (budget key, divisor that converts the field to the budget's unit, label)
MEASURES = {
    'title_ms': ('title_ms', 1, 'Title visible (ms)'),
    'fcp_ms': (None, 1, 'First contentful paint (ms)'),
    'data_before_title': ('data_before_title_kb', 1000, 'Data before title (KB, gzip)'),
    'script_before_title': ('script_before_title_kb', 1000, 'Script before title (KB)'),
    'peak_heap': ('peak_heap_mb', 1_000_000, 'Peak JS heap (MB)'),
}


def load_budgets(path=BUDGETS):
    return tomllib.loads(Path(path).read_text(encoding='utf-8'))


def gzip_size(data):
    return len(gzip.compress(data, 6))


def median(values):
    values = [v for v in values if v is not None]
    return statistics.median(values) if values else None


def percentile(values, fraction):
    """Nearest-rank percentile; None when there are no values."""
    values = sorted(v for v in values if v is not None)
    if not values:
        return None
    rank = max(1, math.ceil(fraction * len(values)))
    return values[rank - 1]


def summarize_loads(loads):
    """Median of each measure across repeated loads of one page."""
    return {field: median([load.get(field) for load in loads]) for field in MEASURES}


def keystroke_latencies(keys, mutations):
    """For each keydown time, the time until the last results change before the next keydown.

    A keystroke that changes nothing before the next keystroke has no latency (None).
    """
    latencies = []
    for index, start in enumerate(keys):
        end = keys[index + 1] if index + 1 < len(keys) else math.inf
        changes = [m for m in mutations if start <= m < end]
        latencies.append(max(changes) - start if changes else None)
    return latencies


def over_budget(build, profile, field, value, budgets):
    """Return the budget a value exceeds, or None. Only lsg values are checked."""
    budget_key, divisor, _ = MEASURES.get(field, (field, 1, field))
    if build != 'lsg' or value is None or budget_key is None:
        return None
    limit = budgets.get(profile, {}).get(budget_key)
    if limit is None or value / divisor <= limit:
        return None
    return f'{budget_key} = {limit}'


def completion_latencies(keys, values, completions):
    """For each keydown, the time until a completed result render shows its input or a later one.

    `values[i]` is the input after key `i`; `completions` holds (time, input) pairs. A key whose
    input was replaced before any render showed it counts from the first render of a longer input
    that starts with it, since that render also answers it. A key no render answered has None.
    """
    latencies = []
    for start, value in zip(keys, values):
        done = [time for time, shown in completions if time >= start and shown.startswith(value)]
        latencies.append(min(done) - start if done else None)
    return latencies


def keystroke_fetches(keys, resources, shard_pattern):
    """For each keydown, the bytes of the files requested before the next key, and whether one is a shard.

    `resources` holds (start time, URL, transferred bytes) triples.
    """
    fetches = []
    for index, start in enumerate(keys):
        end = keys[index + 1] if index + 1 < len(keys) else math.inf
        found = [(url, size) for time, url, size in resources if start <= time < end]
        fetches.append({'bytes': sum(size for _, size in found),
                        'shard': any(shard_pattern.search(url) for url, _ in found)})
    return fetches
