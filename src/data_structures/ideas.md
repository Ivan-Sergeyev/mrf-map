# Ideas

## Labeled graph

- every edge has a label (0..k)
- fixing label k -> same interface as in PetGarph (aka k independent graphs with the same vertex set)
- can also work with the whole graph (aka all edges together, regardless of label)
- some edges may have several labels at once, need to be able to handle that too

## Split CFN

- splitting only creates new labels, never deletes old ones
- graph edges and messages can be appended to existing structures (synchronized by edge index, so OK)
- cost function values are either old ones or +inf, so can store the same CFN
- after split label -> (ref to old unsplit label, ref to splitting source)
- if full split, then refs to splitting sources are exhaustive for source vertex
- can recurse the construction, so split label -> (ref to prev level label, ref to splitting source)
- (label, label) means label is unsplit (if it never appears as an "old label" reference) or serves as default (if there is an arrow to it)
- deep splits are equivalent to (orig label, assignment to vars that served as split sources)
- if depth is small, no need to shorten arrows, otherwise can apply Tarjan strat
- after a split, need to reassign/remap messages
