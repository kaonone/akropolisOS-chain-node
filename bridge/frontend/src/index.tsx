import React from 'react';
import ReactDOM from 'react-dom';

import { Root } from 'core/Root';

function render(component: React.ReactElement): void {
  ReactDOM.render(component, window.document.getElementById('root'));
}

render(<Root />);
