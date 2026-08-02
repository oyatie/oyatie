def _four_digit_ordinal(value):
    if value < 0 or value > 9999:
        fail("corpus YAML shard ordinal must be between 0 and 9999")
    if value < 10:
        return "000%d" % value
    if value < 100:
        return "00%d" % value
    if value < 1000:
        return "0%d" % value
    return "%d" % value


def corpus_yaml_facts_shards(srcs, shard_size):
    if shard_size <= 0:
        fail("corpus YAML shard size must be positive")
    if not srcs:
        fail("corpus YAML shard input must not be empty")

    package = native.package_name()
    ordered = sorted(srcs)
    for start in range(0, len(ordered), shard_size):
        ordinal = start // shard_size
        if ordinal == 0:
            name = "corpus-yaml-facts"
            out = "yaml-facts.json"
        else:
            suffix = _four_digit_ordinal(ordinal)
            name = "corpus-yaml-facts-shard-" + suffix
            out = "yaml-facts-shard-" + suffix + ".json"
        native.genrule(
            name = name,
            srcs = ordered[start:start + shard_size],
            out = out,
            cmd = "$(exe //governance/corpus/extract:yaml-facts) --target root//%s:%s --prefix %s --out $OUT $SRCS" % (package, name, package),
        )
