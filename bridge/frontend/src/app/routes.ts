import build from 'build-route-tree';

const rawTree = {
  demo: null,
  ethereum: null,
  substrate: null,
};

export const routes = build(rawTree);
