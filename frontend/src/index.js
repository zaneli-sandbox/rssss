import './main.css';
import { Elm } from './Main.elm';
import registerServiceWorker from './registerServiceWorker';

Elm.Main.init({
  node: document.getElementById('root'),
  flags : { backendUrl: process.env.ELM_APP_BACKEND_URL },
});

registerServiceWorker();
