from mip import *


# Set parameters of frustrated cycle
cycle_len = 3
cycle_dom_sizes = 2
cycle_vars = list(range(cycle_len))
cycle_edges = [(i, (i + 1) % cycle_len) for i in cycle_vars]
cycle_doms = [list(range(cycle_dom_sizes)) for i in cycle_vars]
cycle_singletons = {i: {a: 0 for a in cycle_doms[i]} for i in cycle_vars}
cycle_interactions = {
    edge: {a: {b: int((a != b) ^ (edge == cycle_edges[-1]))
    for b in cycle_doms[edge[1]]} for a in cycle_doms[edge[0]]} for edge in cycle_edges
}
# todo: generalize cycle interactions to non-binary labels?

print(cycle_interactions)

# Create empty model
osac_lp = Model(sense=MAXIMIZE, solver_name=CBC)

# Add variables
u = {i: osac_lp.add_var(lb=-float('inf'), name=f'u[{i}]') for i in cycle_vars}
p = {
    edge: {i: {a: osac_lp.add_var(lb=-float('inf'), name=f'p[{edge}][{i}][{a}]')
    for a in cycle_doms[i]} for i in edge} for edge in cycle_edges
}

# Set objective
osac_lp.objective = xsum(u[i] for i in cycle_vars)

# Add constraints
for i in cycle_vars:
    for a in cycle_doms[i]:
        edge_1 = (i, (i + 1) % cycle_len)
        edge_2 = ((cycle_len + i - 1) % cycle_len, i)
        osac_lp += -u[i] + cycle_singletons[i][a] + p[edge_1][i][a] + p[edge_2][i][a] >= 0, f'{i} {a}'

for edge in cycle_edges:
    for a in cycle_doms[edge[0]]:
        for b in cycle_doms[edge[1]]:
            osac_lp += cycle_interactions[edge][a][b] - p[edge][edge[0]][a] - p[edge][edge[1]][b] >= 0, f'{edge} {a} {b}'

# Solve
status = osac_lp.optimize(max_seconds=300)

# Check result
if status == OptimizationStatus.OPTIMAL:
    print(f'found optimal solution of value {osac_lp.objective_value} found')
    for v in osac_lp.vars:
        print(f'{v.name} = {v.x}')
else:
    print(f'optimization status: {status}')
