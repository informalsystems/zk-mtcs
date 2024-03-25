use light_poseidon::{Poseidon, PoseidonHasher};
use ark_bn254::{Fr};

// Compute a merkle tree given an array
// This function now returns a Vec<Vec<Fr>>, with each inner Vec<Fr> being a layer in the Merkle tree.
pub fn compute_merkle_tree(arr: &[Fr]) -> Vec<Vec<Fr>> {
    let mut poseidon = Poseidon::<Fr>::new_circom(2).unwrap();
    if arr.is_empty() {
        unimplemented!() // Not defined for empty arrays
    } else if arr.len() == 1 {
        return vec![vec![arr[0].clone()]];
    }

    let mid = (arr.len() + 1) / 2; // Adjust mid for odd lengths
    let left_tree = compute_merkle_tree(&arr[0..mid]);
    let right_tree = if arr.len() % 2 == 0 {
        compute_merkle_tree(&arr[mid..])
    } else {
        // For odd lengths, duplicate the last part of the left tree.
        compute_merkle_tree(&arr[mid-1..])
    };

    // Combine the left and right trees at each level and add a new root level.
    let mut combined_tree = Vec::new();
    let max_depth = left_tree.len().max(right_tree.len());
    for depth in 0..max_depth {
        let mut layer = Vec::new();
        if depth < left_tree.len() {
            layer.extend_from_slice(&left_tree[depth]);
        }
        if depth < right_tree.len() && depth < left_tree.len() {
            layer.extend(right_tree[depth].iter().cloned());
        }
        combined_tree.push(layer);
    }

    // Compute and add the new root from the last elements of the left and right trees.
    let new_root = poseidon.hash(&[
	*combined_tree.last().unwrap().first().unwrap(),
	*combined_tree.last().unwrap().last().unwrap()
    ]).unwrap();
    combined_tree.push(vec![new_root]);

    combined_tree
}

// Compute just the root
pub fn compute_merkle_tree2(arr: &[Fr]) -> Fr {
    let mut poseidon = Poseidon::<Fr>::new_circom(2).unwrap();
    match arr.len() {
        0 => unimplemented!(),
        1 => arr[0],
        _ => {
            let mid = arr.len() / 2;
            let left = compute_merkle_tree2(&arr[0..mid]);
            let right = if arr.len() % 2 == 0 {
                compute_merkle_tree2(&arr[mid..])
            } else {
                // For odd number of elements, duplicate the last one
                compute_merkle_tree2(&[arr[mid-1].clone(), arr[mid-1].clone()])
            };
            poseidon.hash(&[left, right]).unwrap()
        }
    }
}

// Compute the path
pub fn compute_merkle_path(tree: &Vec<Vec<Fr>>, leaf_index: usize) -> Vec<Fr> {
    let mut path = Vec::new();
    let mut current_index = leaf_index;

    for layer in tree.iter().take(tree.len() - 1) { // Exclude the root
        let pair_index = if current_index % 2 == 0 { current_index + 1 } else { current_index - 1 };

        // Check if the pair index is within the current layer's bounds
        if pair_index < layer.len() {
            path.push(layer[pair_index].clone());
        } else {
            // If not, it means the layer was extended due to an odd number of elements,
            // so we add the last element of the layer as its own pair.
            path.push(layer.last().unwrap().clone());
        }

        current_index /= 2; // Move up to the next layer
    }

    path
}

// Merkle proof
pub fn compute_merkle_root(leaf: Fr, index: usize, hash_path: &[Fr]) -> Fr {
    let mut poseidon = Poseidon::<Fr>::new_circom(2).unwrap();
    let n = hash_path.len();
    let mut current_index = index;
    let mut current = leaf;
    for i in 0..n {
        let path_bit = current_index % 2 != 0;
        let (hash_left, hash_right) = if path_bit {
            (hash_path[i], current)
        } else {
            (current, hash_path[i])
        };
        current = poseidon.hash(&[hash_left, hash_right]).unwrap();
        current_index /= 2; // Move up to the next layer
    }
    current
}

// Update an element of the tree
pub fn update_merkle_tree(tree: &mut Vec<Vec<Fr>>, leaf_index: usize, new_value: Fr) -> Fr {
    let mut poseidon = Poseidon::<Fr>::new_circom(2).unwrap();    
    // Step 1: Update the leaf node
    tree[0][leaf_index] = new_value;

    // Step 2: Update the path from the updated leaf to the root
    let mut current_index = leaf_index;
    for depth in 0..tree.len() - 1 { // Exclude the root itself
        let pair_index = if current_index % 2 == 0 { current_index + 1 } else { current_index - 1 };
        
        // Handle the case where the layer size is odd and the current node is the last one
        let sibling_exists = pair_index < tree[depth].len();
        let parent_index = current_index / 2;

        // Compute the new parent node. If there's no sibling (odd number of nodes), use the current node twice.
        let new_parent = if sibling_exists {
            let sibling = &tree[depth][pair_index];
            if current_index % 2 == 0 {
                poseidon.hash(&[tree[depth][current_index], *sibling]).unwrap()
            } else {
                poseidon.hash(&[*sibling, tree[depth][current_index]]).unwrap()
            }
        } else {
            poseidon.hash(&[tree[depth][current_index], tree[depth][current_index]]).unwrap()
        };

        // Update the parent node in the next layer
        tree[depth + 1][parent_index] = new_parent;
        current_index = parent_index;
    }
    tree.last().unwrap()[0]
}