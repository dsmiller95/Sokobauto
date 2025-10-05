import { Graph, GraphConfigInterface } from "@cosmos.gl/graph";
import React from "react";
import { getColorsByMaxTargets } from "../helpers";
import { GraphData } from "../models";

interface Props {
  data: GraphData;
}

const spaceSize = 8192;

function applyGraphData(graph: Graph, data: GraphData) {
  const maxOnTargets = Math.max(...data.nodes.map(n => n.on_targets));
  const colorsByTargets = getColorsByMaxTargets(maxOnTargets);
  

  const idToIndex: Record<number, number> = {};
  const pointPositions: number[] = [];
  const pointColors: number[] = [];
  let nodeIndex = 0;
  for (const node of data.nodes) {
    idToIndex[node.id] = nodeIndex;
    nodeIndex++;

    const x = Math.random() * spaceSize/2;
    const y = Math.random() * spaceSize/2;
    pointPositions.push(x, y);
    const color = colorsByTargets[node.on_targets] ?? [255, 0, 0, 255];
    pointColors.push(...color);
  }

  const links: number[] = [];
  for (const link of data.links) {
    const sourceIndex = idToIndex[link.source];
    const targetIndex = idToIndex[link.target];
    if (sourceIndex === undefined || targetIndex === undefined) {
      console.warn(`Link with unknown source or target: ${link.source} -> ${link.target}`, link);
      continue;
    }

    links.push(sourceIndex, targetIndex);
  }

  graph.setPointPositions(new Float32Array(pointPositions));
  graph.setPointColors(new Float32Array(pointColors));
  graph.setLinks(new Float32Array(links));
}

function createGraph(div: HTMLDivElement): Graph {
  const config: GraphConfigInterface = {
    spaceSize: spaceSize,
    // scalePointsOnZoom: true,
    pointSize: 4,
    simulationFriction: 0.5, // keeps the graph inert
    simulationGravity: 0, // disables the gravity force
    // simulationRepulsion: 0.5, // increases repulsion between points
    simulationDecay: 50000,
    fitViewOnInit: true, // fit the view to the graph after initialization
    fitViewDelay: 1000, // wait 1 second before fitting the view
    fitViewPadding: 0.3, // centers the graph with a padding of ~30% of screen
    // simulationRepulsion: 3,
    simulationCluster: 0.1,
    rescalePositions: false, // rescale positions, useful when coordinates are too small
    enableDrag: true, // enable dragging points
    simulationRepulsionFromMouse: 1000,
    enableRightClickRepulsion: true,
    // onClick: (pointIndex) => { console.log('Clicked point index: ', pointIndex) },
    /* ... */
  };

  const graph = new Graph(div, config);
  return graph;
}

const CosmosGraph = ({data}: Props) => {
  const divRef = React.useRef<HTMLDivElement>(null);
  const graphRef = React.useRef<Graph>(null);

  React.useEffect(() => {
    if (!divRef.current) return;
    if (graphRef.current) return;

    graphRef.current = createGraph(divRef.current);
    applyGraphData(graphRef.current, data);
    graphRef.current.render();

    return () => {
      graphRef.current?.destroy();
      graphRef.current = null;
    };
  }, [divRef.current, data]);

  return <div ref={divRef}></div>;
};


export default CosmosGraph;