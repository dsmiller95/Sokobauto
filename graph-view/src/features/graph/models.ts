interface GraphData {
    nodes: GraphNode[];
    links: GraphLink[];
}

interface GraphNode {
    id: number;
    on_targets: number;
}

interface GraphLink {
    source: number;
    target: number;
}