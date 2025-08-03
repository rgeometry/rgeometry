DOI: https://doi.org/10.1145/322139.322142

The algorithm presented in "An Optimal Algorithm for Finding the Kernel of a Polygon" by D. T. Lee and F. P. Preparata is a linear-time method for computing the kernel of a simple polygon with $ n $ vertices. Below is a concise summary of the algorithm, its key ideas, and its optimality:

# Problem Overview:
- The kernel $ K(P) $ of a simple polygon $ P $ is the set of all interior points from which every point in $ P $ (especially all vertices) is visible — i.e., the segment from the kernel point to any vertex lies entirely within $ P $.
- Equivalently, the kernel is the intersection of all half-planes lying to the left of each directed edge of the polygon (since edges are oriented counterclockwise).
- While intersecting $ n $ arbitrary half-planes takes $ O(n \log n) $ time, this algorithm exploits the ordered structure of the polygon’s edges to achieve optimal $ O(n) $ time.

# Key Observations:
1. Ordered Edges: The edges of $ P $ are given in order along the boundary, so the corresponding half-planes are also ordered.
2. Kernel as Convex Polygon: The kernel, if non-empty, is a convex polygon (possibly unbounded) formed by the intersection of these half-planes.
3. Sequential Intersection: Instead of processing generic half-planes, the algorithm incrementally constructs intermediate polygons $ K_i $, the intersection of the first $ i+1 $ half-planes.

# Algorithm Summary:

## Initialization:
- Start with $ K_1 $: the intersection of the half-planes defined by the first two edges $ e_0 $ and $ e_1 $.

- This is an infinite wedge: $ K_1 = A_{e_1}v_0e_0A $, bounded by two rays extending from $ v_0 $.

- Define special support vertices $ F_i $ and $ L_i $: the "farthest" points on the current kernel $ K_i $ along two supporting lines through the initial vertex $ v_0 $. These help in detecting failure conditions.

## Iterative Step (from $ K_i $ to $ K_{i+1} $):
For each vertex $ v_i $ (and edge $ e_{i+1} $), update $ K_i $ by intersecting it with the half-plane $ H_{i+1} $ to the left of edge $ e_{i+1} $.

Two main cases based on the type of vertex $ v_i $:

1. Reflex Vertex $ v_i $:
    - The interior angle > 180°. The new edge may truncates the current kernel.
    - Intersect $ K_i $ with the new half-plane.
    - Traverse the boundary of $ K_i $ clockwise and counterclockwise from $ F_i $ to find where the new edge's line intersects $ K_i $.
    - Update $ K_{i+1} $ by clipping $ K_i $ accordingly, possibly making it bounded or unbounded.

2. Convex Vertex $ v_i $:
    - Interior angle < 180°.
    - Often requires fewer changes; sometimes $ K_{i+1} = K_i $, but still need to update support points $ F_{i+1} $, $ L_{i+1} $.
- In both cases, the algorithm maintains $ F_i $ and $ L_i $ — supporting points used to check convexity and visibility constraints.

## Termination Conditions:

- If during the update no intersection with the new half-plane can be found (e.g., the scan reaches both ends without crossing), the kernel becomes empty; terminate and return $ K(P) = \emptyset $.
- Additionally, a geometric test checks whether the next edge $ e_{i+1} $ crosses certain supporting lines ("$ f $" or "$ l $") from the initial vertex $ v_0 $, indicating that the polygon boundary has "wrapped around" — implying $ K(P) = \emptyset $ even if local intersection exists. This prevents unnecessary work.

## Efficiency Enhancements:
- A critical insight: each edge of the intermediate kernel $ K_i $ can be checked at most once per vertex due to monotonic advancement of the scan pointers.
- The total number of operations over all steps is $ O(n) $ because:
    - Each intersection operation removes edges that won’t be revisited.
    - Scanning for support vertices uses a "pointer" mechanism that advances monotonically.
    - Every edge of $ P $ contributes at most a constant number of visits.

# Correctness and Complexity:
- Lemma 1: By induction, $ K_i $ is the intersection of the first $ i+1 $ left half-planes — hence $ K_{n-1} = K(P) $ at the end.
- Theorem 1 & Corollary 1: Early termination tests ensure no redundant computation when kernel becomes unwrappable.
- Time Complexity: $ O(n) $ — each edge is processed a constant number of times in scanning and clipping.
- Optimality: Since every edge must be read at least once, $ \Omega(n) $ is a lower bound — thus $ O(n) $ is optimal.

# Conclusion:

The Lee–Preparata algorithm efficiently computes the kernel of a simple polygon by:

Exploiting the ordered sequence of edges,
Maintaining a dynamic convex polygonal kernel,
Using support vertices $ F_i $, $ L_i $ to guide updates and detect emptiness,
Guaranteeing linear time performance $ O(n) $, which is optimal.
It improves upon earlier $ O(n \log n) $ half-plane intersection methods by specifically using the geometric ordering inherent in polygons.

Significance:
A classic result in computational geometry.
Demonstrates how structural properties (like edge ordering in polygons) can be leveraged to beat general-case complexity bounds.
Foundational for visibility problems, art gallery algorithms, and polygon decomposition.

# ✅ Summary in One Sentence:

The Lee–Preparata algorithm computes the kernel of a simple polygon in optimal $ O(n) $ time by incrementally intersecting ordered edge half-planes and using support vertices to maintain and update the kernel efficiently.
