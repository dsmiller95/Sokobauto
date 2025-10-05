import { getRgbaColor, Graph, GraphConfigInterface } from '@cosmos.gl/graph';
import { createFileRoute } from '@tanstack/react-router'
import React from 'react'

const spaceSize = 8192;

async function getGraphData(): Promise<GraphData> {
  const response = await fetch('/graph/data-json')
  return await response.json()
}

const minColor = [117/255, 70/255, 63/255, 255/255];
const maxColor = [7/255, 14/255, 227/255, 255/255];

function applyGraphData(graph: Graph, data: GraphData) {
  const maxOnTargets = Math.max(...data.nodes.map(n => n.on_targets));
  const colorsByTargets = Array(maxOnTargets + 1).fill(0).map((_, i) => {
    const t = maxOnTargets === 0 ? 0 : i / maxOnTargets;
    return minColor.map((minC, index) => {
      const maxC = maxColor[index];
      return minC * (1 - t) + maxC * t;
    });
  });
  

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
      console.warn(`Link with unknown source or target: ${link.source} -> ${link.target}`);
      continue;
    }

    links.push(sourceIndex, targetIndex);
  }

  graph.setPointPositions(new Float32Array(pointPositions));
  graph.setPointColors(new Float32Array(pointColors));
  graph.setLinks(new Float32Array(links));
}

export const Route = createFileRoute('/graph/view')({
  component: RouteComponent,
})

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
    onMouseMove: (idx, pos, event) => {
      // only works in chromium. this library sucks.
      console.log('is right click?', event.which === 3);
    }
    /* ... */
  };

  const graph = new Graph(div, config);
  return graph;
}

function RouteComponent() {
  const divRef = React.useRef<HTMLDivElement>(null);
  const graphRef = React.useRef<Graph>(null);
  const renderCount = React.useRef(0);
  renderCount.current++;

  const [isChecked, setChecked] = React.useState(false);
  const [graphData, setGraphData] = React.useState<GraphData | null>(null);

  React.useEffect(() => {
    if (!divRef.current) return;
    if (graphRef.current) return;
    divRef.current.innerHTML = `<h1>Graph View ${renderCount.current}</h1>`
    graphRef.current = createGraph(divRef.current);
  }, [divRef.current])

  React.useEffect(() => {
    getGraphData().then((data) => {
      setGraphData(data);
      if (!graphRef.current) {
        console.error('Graph not initialized yet');
        return;
      }
      applyGraphData(graphRef.current, data);
      graphRef.current.render();
    });
  }, [divRef.current]);

  return (<div>
    <div ref={divRef}>Hello "/graph/view"!</div>
    <input type="checkbox" checked={isChecked} onChange={(e) => setChecked(e.target.checked)} />
  </div>)
}
