import { LitElement, html, css, customElement, property } from "lit-element";

const STR_CONTINUE = "Continue";

@customElement("module-footer-continue-button")
export class _ extends LitElement {
    static get styles() {
        return [css``];
    }

    @property({ type: Boolean })
    enabled: boolean = false;

    render() {
        const { enabled } = this;

        const pointer = enabled ? "initial" : "none";

        return html`
            <button-rect
                .disabled=${!enabled}
                style="pointer-events: ${pointer}"
                size="regular"
                iconAfter="arrow"
                @click=${() => {
                    if (enabled) {
                        this.dispatchEvent(new Event("next"));
                    }
                }}
                >${STR_CONTINUE}
            </button-rect>
        `;
    }
}
