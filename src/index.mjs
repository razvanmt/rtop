import { h, render } from 'https://esm.sh/preact';
import htm from 'https://esm.sh/htm';

const html = htm.bind(h);

let i = 0;

function App(props) {
    return html`
    <div>
    ${props.cpus.map((cpu) => {
        return html `<div>${cpu.toFixed(2)}% usage</div> `;
    })}
    </div>
    `;
}

setInterval(async() => {
    let response = await fetch('/api/cpus');
    if (response.status !== 200){
        throw new Error(`Error! Status code: ${response.status}`);
    }
    let json = await response.json();
    
    const app = h("pre", null, JSON.stringify(json, null, 2));

    render(html`<${App} cpus=${json}></${App}>`, document.body);
}, 100);