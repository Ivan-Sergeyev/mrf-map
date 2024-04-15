import itertools as it
import math
import unittest

from vcsp import VCSPInstance


class SimpleFrustratedCycle(VCSPInstance):
    # Frustrated cycle
    def __init__(self, length: int, domain_sizes: int):
        self.len = length
        vars = list(range(self.len))
        doms = {var: list(range(domain_sizes)) for var in vars}
        edges = [self.edge_next(var) for var in vars]
        cost_none = {(): {(): 0}}
        cost_vars = {(var,): {(label,): 0 for label in doms[var]} for var in vars}
        cost_edges = {edge: {labels: int((labels[0] != labels[1]) ^ (edge == edges[-1]))
                      for labels in it.product(*(doms[var] for var in edge))} for edge in edges}
        costs = cost_none | cost_vars | cost_edges
        super().__init__(vars, doms, costs)

    def edge_next(self, i):
        return (i, (i + 1) % self.len)

    def edge_prev(self, i):
        return ((self.len + i - 1) % self.len, i)


class SimpleFrustratedCycleOneSplit(VCSPInstance):
    # Frustrated cycle with a split of one (destination) variable based on labels of another (source) variable
    def __init__(self, length: int, domain_sizes: int, split_var_src: int, split_var_dst: int):
        assert(0 <= split_var_src and split_var_src < length and
               0 <= split_var_dst and split_var_dst < length)

        sfc = SimpleFrustratedCycle(length, domain_sizes)

        doms = {var: [(label, label) for label in sfc.doms[var]] if var != split_var_dst else
                     [label_pair for label_pair in it.product(sfc.doms[split_var_src], sfc.doms[split_var_dst])]
                for var in sfc.vars}

        costs = {edge: {labels: math.inf if (split_var_dst in edge and split_var_src in edge and
                        labels[0][0] != labels[1][0]) else sfc.costs[edge][(labels[0][1], labels[1][1])]
                 for labels in it.product(*(doms[var] for var in edge))} for edge in sfc.sets}

        super().__init__(sfc.vars, doms, costs)


class SimpleFrustratedCycleCompleteSplit(VCSPInstance):
    # Frustrated cycle where all variables are split basd on labels of one variable
    def __init__(self, length: int, domain_sizes: int, split_var: int):
        assert(0 <= split_var and split_var < length)

        sfc = SimpleFrustratedCycle(length, domain_sizes)

        doms = {var: [(label, label) for label in sfc.doms[split_var]] if var == split_var else
                     [label_pair for label_pair in it.product(sfc.doms[split_var], sfc.doms[var])] for var in sfc.vars}

        costs = {(): {(): 0}} | \
                {(var,): {(label,): 0 for label in doms[var]} for var in sfc.vars} | \
                {edge: {labels: sfc.costs[edge][(labels[0][1], labels[1][1])]
                        if labels[0][0] == labels[1][0] else math.inf
                        for labels in it.product(*(doms[var] for var in edge))} for edge in sfc.sets}

        super().__init__(sfc.vars, doms, costs)


class TestFrustratedCycleVariations(unittest.TestCase):
    # todo: test constructors

    def test_sfc_osac(self):
        # Solve simple frustrated cycle
        sfc = SimpleFrustratedCycle(3, 2)
        sfc_osac_costs = sfc.get_osac_costs()
        self.assertEqual(sfc_osac_costs, sfc.costs)

    def test_sfc_os_osac(self):
        # Solve frustrated cycle with one variable splitting (split 1 on 0)
        sfc_os = SimpleFrustratedCycleOneSplit(3, 2, 0, 1)
        sfc_os_osac_costs = sfc_os.get_osac_costs()
        self.assertEqual(sfc_os_osac_costs, sfc_os.costs)

    def test_sfc_cs_osac(self):
        # Solve frustrated cycle with complete variable splitting (split both 1 and 2 on 0)
        sfc_cs = SimpleFrustratedCycleCompleteSplit(3, 2, 0)
        sfc_cs_osac_costs = sfc_cs.get_osac_costs()
        output_values = [val for d in sfc_cs_osac_costs.costs.values() for val in d.values()]
        expected_values = [
            1.0,  # empty
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,  # unary
            0.0, 0.0, math.inf, math.inf, math.inf, math.inf, 0.0, 0.0,  # edge (0, 1)
            0.0, 0.0, math.inf, math.inf, 2.0, 0.0, math.inf, math.inf,
            math.inf, math.inf, 0.0, 2.0, math.inf, math.inf, 0.0, 0.0,  # edge (1, 2)
            0.0, math.inf, 0.0, math.inf, math.inf, 0.0, math.inf, 0.0  # edge (2, 0)
        ]
        self.assertEqual(output_values, expected_values)


if __name__ == '__main__':
    unittest.main()
