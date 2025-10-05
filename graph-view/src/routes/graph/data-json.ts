import fs from 'node:fs'
import { createFileRoute } from '@tanstack/react-router'

const filePath = '../RulesEngine/exports/state_graph.json';

const defaultGraphData = {
  "nodes": [
    {
      "id": 1,
      "on_targets": 0
    },
    {
      "id": 2,
      "on_targets": 0
    },
    {
      "id": 3,
      "on_targets": 1
    }
  ],
  "links": [
    {
      "source": 1,
      "target": 2,
    },
    {
      "source": 2,
      "target": 3,
    }
  ]
};

export const Route = createFileRoute('/graph/data-json')({
  server: {
    handlers: {
      GET: async () => {
        const responseData = await fs.promises.readFile(filePath, 'utf-8').catch(() =>
              JSON.stringify(defaultGraphData)
            );
        return new Response(responseData, {
          headers: {
            'Content-Type': 'application/json',
          },
        });
      },
    },
  },
})
