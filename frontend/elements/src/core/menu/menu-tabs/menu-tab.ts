import { LitElement, html, css, customElement, property } from "lit-element";

@customElement("menu-tab")
export class _ extends LitElement {
    static get styles() {
        return [
            css`
                :host {
                    display: inline-grid;
                    border-top-left-radius: 10px;
                    border-top-right-radius: 10px;
                    padding: 8px 10px;
                    cursor: pointer;
                }
                :host([active]) {
                    background-color: #e9eff8;
                }
            `,
        ];
    }

    @property({ type: Boolean, reflect: true })
    active: boolean = false;

    render() {
        return html` <slot></slot> `;
    }
}
