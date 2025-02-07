extern crate polars as pl;

use kubegraph_api::{graph::GraphData, problem::ProblemSpec, solver::NetworkSolver as _};
use kubegraph_solver_ortools::NetworkSolver;
use pl::{
    df,
    frame::DataFrame,
    lazy::{dsl, frame::IntoLazy},
};

#[::tokio::test]
async fn solver_simple() {
    // Step 1. Define edges
    let edges = df!(
        "src"       => [  0],
        "sink"      => [  1],
        "capacity"  => [ 20],
        "unit_cost" => [  1],
    )
    .expect("failed to create edges dataframe");

    // Step 2. Define edges
    let nodes = df!(
        "name"      => [  0,   1],
        "capacity"  => [ 20,  10],
        "supply"    => [ 20,   0],
        "unit_cost" => [  5,   0],
    )
    .expect("failed to create nodes dataframe");

    // Step 3. Define a graph
    let graph = GraphData { edges, nodes };

    // Step 4. Define a problem
    let problem = ProblemSpec {
        verbose: true,
        ..Default::default()
    };

    // Step 5. Define a solver
    let solver = NetworkSolver::default();

    // Step 6. Optimize the graph
    let optimized_graph: GraphData<DataFrame> = solver
        .solve(graph, &problem)
        .await
        .expect("failed to optimize the graph")
        .try_into()
        .expect("failed to collect graph");
    let GraphData {
        edges: mut optimized_edges,
        nodes: optimized_nodes,
    } = optimized_graph;

    let edges_flow = optimized_edges.column("flow").unwrap();
    let edges_unit_cost = optimized_edges.column("unit_cost").unwrap();

    let edges_cost = (edges_flow * edges_unit_cost)
        .expect("failed to get edges cost")
        .with_name("cost".into());
    optimized_edges
        .with_column(edges_cost)
        .expect("failed to insert edge cost column");

    println!();
    println!("{}", &optimized_nodes);
    println!("{}", &optimized_edges);

    let optimized_edges = optimized_edges.clone();
    let get_arc_cost = |src, sink| -> u64 {
        optimized_edges
            .lazy()
            .filter(dsl::col("src").eq(src).and(dsl::col("sink").eq(sink)))
            .collect()
            .expect("failed to search an edge")
            .column("cost")
            .unwrap()
            .get(0)
            .expect("no such edge")
            .try_extract()
            .expect("failed to extract edge cost value")
    };

    assert_eq!(get_arc_cost(0, 1), 10);
}
