import math
from mip import *


def edge_next(i, cycle_len):
    return (i, (i + 1) % cycle_len)


def edge_prev(i, cycle_len):
    return ((cycle_len + i - 1) % cycle_len, i)


# Set parameters of frustrated cycle
cycle_len = 3
cycle_dom_sizes = 2
cycle_vars = list(range(cycle_len))
cycle_edges = [edge_next(i, cycle_len) for i in cycle_vars]
cycle_doms = [list(range(cycle_dom_sizes)) for i in cycle_vars]
cycle_singletons = {i: {a: 0 for a in cycle_doms[i]} for i in cycle_vars}
cycle_interactions = {
    edge: {a: {b: int((a != b) ^ (edge == cycle_edges[-1]))
    for b in cycle_doms[edge[1]]} for a in cycle_doms[edge[0]]} for edge in cycle_edges
}
# todo: generalize cycle interactions to non-binary labels?

# Split on variable 0
split_len = cycle_len
split_vars = cycle_vars
split_edges = cycle_edges
split_doms = [[(a, a) for a in cycle_doms[0]]] + [[(a, b) for a in cycle_doms[0] for b in cycle_doms[i]] for i in range(1, cycle_len)]
split_singletons = {i: {a: cycle_singletons[i][a[1]] for a in split_doms[i]} for i in split_vars}
split_interactions = {
    edge: {a: {b: cycle_interactions[edge][a[1]][b[1]] if a[0] == b[0] else math.inf
    for b in split_doms[edge[1]]} for a in split_doms[edge[0]]} for edge in split_edges
}

# Create empty model
osac_lp = Model(sense=MAXIMIZE, solver_name=CBC)

# Add variables
u = {i: osac_lp.add_var(lb=-float('inf'), name=f'u[{i}]') for i in split_vars}
p = {
    edge: {i: {a: osac_lp.add_var(lb=-float('inf'), name=f'p[{edge}][{i}][{a}]')
    for a in split_doms[i]} for i in edge} for edge in split_edges
}

# Set objective
osac_lp.objective = xsum(u[i] for i in split_vars)

# Add constraints
for i in split_vars:
    for a in split_doms[i]:
        edge_1 = edge_next(i, split_len)
        edge_2 = edge_prev(i, split_len)
        osac_lp += split_singletons[i][a] - u[i] + p[edge_1][i][a] + p[edge_2][i][a] >= 0, f'{i} {a}'

for edge in split_edges:
    for a in split_doms[edge[0]]:
        for b in split_doms[edge[1]]:
            if split_interactions[edge][a][b] == math.inf:
                # print(f'Remove redundant constraint for {edge} {a} {b}')
                continue
            osac_lp += split_interactions[edge][a][b] - p[edge][edge[0]][a] - p[edge][edge[1]][b] >= 0, f'{edge} {a} {b}'

# Solve
status = osac_lp.optimize(max_seconds=300)

# Check result
if status == OptimizationStatus.OPTIMAL:
    print(f'found optimal solution of value {osac_lp.objective_value} found')
    for v in osac_lp.vars:
        print(f'{v.name} = {v.x}')
else:
    print(f'optimization status: {status}')
