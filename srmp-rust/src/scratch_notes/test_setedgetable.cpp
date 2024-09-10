#include <string.h>
#include <iostream>

void SetEdgeTable_orig(int nA, int* A, int* AK, int nB, int* B, int* BK, int* table)
{
	int i, j, b=0, k=0;
	int K_array[nB];
	int labeling[nB];
	for (i=0; i<nB; i++)
	{
		K_array[i] = 1;
		for (j=nA-1; A[j] != B[i]; j--) K_array[i] *= AK[j];
	}
	memset(labeling, 0, nB*sizeof(int));
	table[0] = 0;
	while ( 1 )
	{
		for (i=nB-1; i>=0; i--)
		{
			if (labeling[i] < BK[i] - 1) break;
			k -= labeling[i]*K_array[i];
			labeling[i] = 0;
		}
		if (i<0) break;
		labeling[i] ++;
		k += K_array[i];
		table[++b] = k;
	}
}

void SetEdgeTable_edit(int nA, int* A, int* AK, int nB, int* B, int* BK, int* table)
{
    // Args:
    // - nA: number of variables in factor A
    // - A: list of variable identifiers in A (sorted in increasing order)
    // - AK: list of dimensions of variables in A
    // - nB: number of variables in factor B
    // - B: list of variable identifiers in B (sorted in increasing order)
    // - BK: list of dimensions of variables in B
    // - table: stride table to fill in (size = product of everything in BK, aka size of function table of B)

	int K_array[nB];
	for (int i = 0; i < nB; i++)
	{
		K_array[i] = 1;
		for (int j = nA - 1; A[j] != B[i]; j--) {
            K_array[i] *= AK[j];
        }
	}

	int labeling_of_B[nB];
	memset(labeling_of_B, 0, nB * sizeof(int));

	table[0] = 0;
    int i = nB - 1;
	int b = 0;
    int k = 0;
    while(true) {
        if (labeling_of_B[i] < BK[i] - 1) {
            // Move to next variable label
            labeling_of_B[i]++;
            k += K_array[i];
            b++;
            table[b] = k;
            i = nB - 1;
        } else {
            // "Carry"
            k -= labeling_of_B[i] * K_array[i];
            labeling_of_B[i] = 0;
            i--;
            if (i < 0) {
                break;
            }
        }
    }
}

void print_array(int size, int* array) {
    for (int i = 0; i < size; i++) {
        std::cout << array[i] << ' ';
    }
    std::cout << '\n';
}

void compare_arrays(int size, int* array1, int* array2) {
    for (int i = 0; i < size; i++) {
        if (array1[i] != array2[i]) {
            std::cout << "Arrays mismatch at position" << i << '\n';
        }
    }
    std::cout << "Arrays match\n";
}

int main() {
    // Set up example
    int num_variables = 3;
    int domain_sizes[num_variables] = {5, 4, 3};

    int nA = 3;
    int A[nA] = {0, 1, 2};  // must be sorted

    int nB = 2;
    int B[nB] = {0, 2};  // must be sorted and subset of A

    // Domain size arrays and table size are determined automatically
    int AK[nA];
    for (int i = 0; i < nA; i++) {
        AK[i] = domain_sizes[A[i]];
    }

    int BK[nB];
    int table_size = 1;
    for (int i = 0; i < nB; i++) {
        BK[i] = domain_sizes[B[i]];
        table_size *= BK[i];
    }
    int table_orig[table_size];
    int table_edit[table_size];

    // Run original implementation
    SetEdgeTable_orig(nA, A, AK, nB, B, BK, table_orig);
    print_array(table_size, table_orig);

    // Run edited implementation
    SetEdgeTable_edit(nA, A, AK, nB, B, BK, table_edit);
    print_array(table_size, table_edit);

    compare_arrays(table_size, table_orig, table_edit);

    return 0;
}