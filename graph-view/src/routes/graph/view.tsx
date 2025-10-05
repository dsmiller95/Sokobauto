import CosmosGraph from '@/features/graph/components/CosmosGraph';
import ViewForceGraph3D from '@/features/graph/components/ViewForceGraph3D';
import { GraphData } from '@/features/graph/models';
import { createFileRoute } from '@tanstack/react-router'
import React from 'react'


async function getGraphData(): Promise<GraphData> {
  const response = await fetch('/graph/data-json')
  return await response.json()
}

export const Route = createFileRoute('/graph/view')({
  component: RouteComponent,
})

enum GraphType {
  Force3D = '3d-force-graph',
  Cosmos = 'cosmos-graph'
}

function RouteComponent() {
  const [graphData, setGraphData] = React.useState<GraphData | null>(null);
  const [graphType, setGraphType] = React.useState<GraphType>(GraphType.Cosmos);

  React.useEffect(() => {
    getGraphData().then((data) => {
      setGraphData(data);
    });
  }, []);

  if(!graphData) {
    return <div>Loading graph data...</div>;
  }


  return (
    <div className='flex flex-col gap-4 p-4'>
      <div className='flex gap-4'>
        <button
          className={`px-4 py-2 rounded ${graphType === GraphType.Cosmos ? 'bg-blue-500 text-white' : 'bg-gray-200'}`}
          onClick={() => setGraphType(GraphType.Cosmos)}
        >
          Cosmos Graph
        </button>
        <button
          className={`px-4 py-2 rounded ${graphType === GraphType.Force3D ? 'bg-blue-500 text-white' : 'bg-gray-200'}`}
          onClick={() => setGraphType(GraphType.Force3D)}
        >
          3D Force Graph
        </button>
      </div>
      <div className='flex-grow'>
        {graphType === GraphType.Cosmos ? (
          <CosmosGraph data={graphData} />
        ) : (
          <ViewForceGraph3D data={graphData} />
        )}
      </div>
    </div>
  );
}
