import build, {getParam} from 'build-route-tree';

const rawTree = {
  demo: null,
  sourceChain: getParam(null),
};

export const routes = build(rawTree);
