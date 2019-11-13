import * as packageJson from '../../package.json';

export function getEnv() {
  const isProduction = process.env.NODE_ENV === 'production';
  const isDevelopment = process.env.NODE_ENV === 'development';
  const forGhPages = true;
  const appVersion = packageJson.version;
  const withHot = !!module.hot && isDevelopment;

  return {
    isProduction,
    isDevelopment,
    forGhPages,
    withHot,
    appVersion,
  };
}
