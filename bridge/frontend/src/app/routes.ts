import build, { getParam } from 'build-route-tree';

const rawTree = {
  demo: null,
  bridge: {
    sourceChain: getParam(null),
  },
  limits: null,
  settings: null,
  validators: null,
};

export const routes = build(rawTree);
