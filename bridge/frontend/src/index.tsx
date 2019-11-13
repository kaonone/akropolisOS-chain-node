import React from 'react';
import ReactDOM from 'react-dom';

import { Root } from 'core/Root';
import { getEnv } from 'core/getEnv';

function render(component: React.ReactElement): void {
  ReactDOM.render(component, window.document.getElementById('root'));
}

render(<Root />);

console.info(`App version is ${getEnv().appVersion}`);
