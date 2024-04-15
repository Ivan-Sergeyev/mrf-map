from __future__ import annotations

from collections.abc import Mapping
from typing import Any, Union

from copy import deepcopy

import inspect
import itertools as it
import math
import unittest

import mip


class VCSPCosts:
    def __init__(self, costs: Mapping[tuple, Mapping[tuple, Union[int, float]]]):
        self.costs = deepcopy(costs)

    def __str__(self):
        return str(self.costs)

    def __getitem__(self, idx):
        return self.costs[idx]

    def unary_project(self, variable: Any, value: Union[int, float]):
        for label, in self.costs[(variable,)]:
            self.costs[(variable,)][(label,)] -= value
        self.costs[()][()] += value
        # todo: correctly process math.inf's?

    def project(self, scope: tuple[Any], variable: Any, label: Any, value: Union[int, float]):
        assert(len(scope) > 1)
        i = scope.index(variable)
        for labels in self.costs[scope]:
            self.costs[scope][labels] -= value * int(labels[i] == label)
        self.costs[(variable,)][(label,)] += value

    def extend(self, variable: Any, label: Any, scope: tuple[Any], value: Union[int, float]):
        assert(len(scope) > 1)
        i = scope.index(variable)
        for labels in self.costs[scope]:
            self.costs[scope][labels] += value * int(labels[i] == label)
        self.costs[(variable,)][(label,)] -= value
        # todo: correctly process math.inf's?

    def bulk_unary_project(self, variables: list[Any],
                           projected_values: Mapping[Any, Union[int, float]]):
        for var in variables:
            self.unary_project(var, projected_values[var])

    def bulk_project_extend(self, scopes: list[tuple[Any]],
                            projected_values: Mapping[tuple, Mapping[Any, Mapping[Any, Union[int, float]]]]):
        for scope in scopes:
            assert(len(scope) > 1)

            for var in scope:
                for (label,) in self.costs[(var,)]:
                    self.costs[(var,)][(label,)] += projected_values[scope][var][label]

            for labels in it.product(*(self.costs[(var,)] for var in scope)):
                labels = tuple(label[0] for label in labels)
                for var, label in zip(scope, labels):
                    self.costs[scope][labels] -= projected_values[scope][var][label]


class VCSPInstance:
    def __init__(
            self,
            variables: list[Any],
            domains: Mapping[Any, Any],
            costs: Mapping[tuple, Mapping[tuple, Union[int, float]]]
    ):
        '''
        Args:
            variables: Variables, given by a list.
            domains: Domains (aka labels) of each variable, given by dict of lists keyed by variables.
            costs: Cost functions, given by dict of dicts of numbers keyed by variable tuples, then label tuples.
                Numbers can be math.inf.
        '''
        self.vars = deepcopy(variables)
        self.doms = {var: deepcopy(domains.get(var, [])) for var in self.vars}
        self.sets = [scope for scope in costs if len(scope) > 1]
        cost_none = {(): {(): costs.get((), {}).get((), 0)}}
        cost_vars = {(var,): {(label,): costs.get((var,), {}).get((label,), 0)
                     for label in self.doms[var]} for var in self.vars}
        cost_sets = {scope: {assignment: costs[scope][assignment]
                     for assignment in costs[scope]} for scope in self.sets}
        self.costs = VCSPCosts(cost_none | cost_vars | cost_sets)

    def __repr__(self):
        return inspect.cleandoc(f'''
            VCSPInstance(
                vars={self.vars},
                domains={self.doms},
                costs={self.costs}
            )
        ''')

    def __str__(self):
        return inspect.cleandoc(f'''
            Variables: {self.vars}
            Domains: {self.doms}
            Cost functions: {self.costs}
        ''')

    def _create_osac_lp(self):
        # Create empty model
        self.osac_lp = mip.Model(sense=mip.MAXIMIZE, solver_name=mip.CBC)

        # Add vars
        self.u = {var: self.osac_lp.add_var(lb=-float('inf'), name=f'u[{var}]') for var in self.vars}
        self.p = {scope: {var: {label: self.osac_lp.add_var(lb=-float('inf'), name=f'p[{scope}][{var}][{label}]')
                  for label in self.doms[var]} for var in scope} for scope in self.sets}

        # Set objective
        self.osac_lp.objective = mip.xsum(self.u[var] for var in self.vars)

        # Add constraints
        for var in self.vars:
            for label in self.doms[var]:
                self.osac_lp += self.costs[(var,)][(label,)] - self.u[var] + \
                    mip.xsum(self.p[scope][var][label] for scope in self.sets if var in scope) >= 0, f'{var} {label}'

        for scope in self.sets:
            for labels in it.product(*(self.doms[var] for var in scope)):
                if self.costs[scope][tuple(labels)] == math.inf:
                    # print(f'Removed redundant constraint {scope} {labels}.')
                    continue
                self.osac_lp += self.costs[scope][labels] - \
                    mip.xsum(self.p[scope][var][label] for var, label in zip(scope, labels)) >= 0, f'{scope} {labels}'

    def get_osac_costs(self) -> VCSPCosts:
        # Create OSAC LP
        self._create_osac_lp()
        self.osac_lp.verbose = 0  # disable solver messages

        # Solve
        status = self.osac_lp.optimize(max_seconds=300)

        # Check result
        if status != mip.OptimizationStatus.OPTIMAL:
            # print(f'Could not solve OSAC LP to optimality. Optimization status: {status}.')
            return None

        if abs(self.osac_lp.objective_value) < 1e-9:
            # print(f'Current instance is OSAC, because OSAC LP has optimal value {self.osac_lp.objective_value}.')
            return self.costs

        # Collect optimal solution
        u_val = {var: self.u[var].x for var in self.vars}
        p_val = {scope: {var: {label: self.p[scope][var][label].x
                 for label in self.doms[var]} for var in scope} for scope in self.sets}

        # # Print optimal solution
        # print(f'Optimal value: {self.osac_lp.objective_value}')

        # for var in self.vars:
        #     print(f'u[{var}] = {u_val[var]}')

        # for scope in self.sets:
        #     for var in scope:
        #         for label in self.doms[var]:
        #             print(f'p[{scope}][{var}][{label}] = {p_val[scope][var][label]}')

        # Apply soft arc consistency operations
        new_costs = deepcopy(self.costs)
        new_costs.bulk_unary_project(self.vars, u_val)
        new_costs.bulk_project_extend(self.sets, p_val)
        # print(f'New costs: {new_costs.costs}')
        return new_costs

    def get_osac_vcsp(self) -> VCSPInstance:
        new_costs = self.get_osac_costs()
        if new_costs is None:
            return None
        return VCSPInstance(self.vars, self.doms, new_costs)


class TestVCSPCosts(unittest.TestCase):
    # todo: implement
    pass


class TestVCSPInstance(unittest.TestCase):
    # todo: implement
    pass


if __name__ == '__main__':
    unittest.main()
