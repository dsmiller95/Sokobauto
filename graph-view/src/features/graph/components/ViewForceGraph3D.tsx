import ForceGraph3D, { ForceGraph3DInstance } from "3d-force-graph";
import React from "react";
import { colorToHex, getColorsByMaxTargets } from "../helpers";
import { GraphData, GraphNode, GraphLink } from "../models";
import { GraphData as ThreeGraphData, NodeObject as ThreeNodeObject, LinkObject as ThreeLinkObject } from "three-forcegraph";

interface Props {
  data: GraphData;
}

type MyGraphNode = ThreeNodeObject & GraphNode;
type MyGraphLink = ThreeLinkObject<MyGraphNode> & GraphLink;
type MyGraphInstance = ForceGraph3DInstance<MyGraphNode, MyGraphLink>;

function createGraph(div: HTMLDivElement, data: GraphData): MyGraphInstance {
  data = JSON.parse(JSON.stringify(data)); // deep clone to avoid mutating. this library is naughty and will mutate.

  const maxOnTargets = Math.max(...data.nodes.map(n => n.on_targets));
  const colorsByTargets = getColorsByMaxTargets(maxOnTargets).map(colorToHex);

  // Cast our data to the extended types
  const graphData: ThreeGraphData<MyGraphNode, MyGraphLink> = {
    nodes: data.nodes,
    links: data.links
  };

  const graph = new ForceGraph3D(div) as unknown as MyGraphInstance;
  graph
    .graphData(graphData)
    .nodeColor(function (node) {
      return colorsByTargets[node.on_targets] ?? [255, 0, 0, 255];
    })
    .nodeId('id')
    .nodeVal('on_targets')
    .nodeLabel('on_targets')
    .nodeAutoColorBy('on_targets')
    .linkSource('source')
    .linkTarget('target')
    .linkDirectionalArrowLength(3.5)
    .linkDirectionalArrowRelPos(1)
    .cooldownTime(10 * 60 * 1000);

  return graph;
}

const ViewForceGraph3D = ({data}: Props) => {
  const divRef = React.useRef<HTMLDivElement>(null);
  const graphRef = React.useRef<MyGraphInstance>(null);

  React.useEffect(() => {
    if (!divRef.current) return;
    if (graphRef.current) return;
    graphRef.current = createGraph(divRef.current, data);
  }, [divRef.current]);

  return <div ref={divRef}></div>;
};


export default ViewForceGraph3D;