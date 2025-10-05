export interface GraphData {
    nodes: GraphNode[];
    links: GraphLink[];
}

export interface GraphNode {
    id: number;
    on_targets: number;
}

export interface GraphLink {
    source: number;
    target: number;
}