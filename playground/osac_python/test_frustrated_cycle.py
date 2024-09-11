import math
import unittest

import frustrated_cycle as fc


class TestFrustratedCycleVariations(unittest.TestCase):
    # todo: test constructors

    def test_sfc_osac(self):
        # Solve simple frustrated cycle
        sfc = fc.FrustratedCycle(3, 2)
        sfc_osac_costs = sfc.get_osac_costs()
        self.assertEqual(sfc_osac_costs, sfc.costs)

    def test_sfc_os_osac(self):
        # Solve frustrated cycle with one variable splitting (split 1 on 0)
        sfc_os = fc.FrustratedCycleOneSplit(3, 2, 0, 1)
        sfc_os_osac_costs = sfc_os.get_osac_costs()
        self.assertEqual(sfc_os_osac_costs, sfc_os.costs)

    def test_sfc_cs_osac(self):
        # Solve frustrated cycle with complete variable splitting (split both 1 and 2 on 0)
        sfc_cs = fc.FrustratedCycleCompleteSplit(3, 2, 0)
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
