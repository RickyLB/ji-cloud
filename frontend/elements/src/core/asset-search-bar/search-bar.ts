import { LitElement, html, css, customElement, property } from "lit-element";

@customElement("asset-search-bar")
export class _ extends LitElement {
    static get styles() {
        return [
            css`
                :host {
                    position: relative;
                    display: flex;
                    font-size: 14px;
                }
                .bar {
                    border-radius: 36px;
                    background-color: #ffffff;
                    width: 100%;
                    display: grid;
                    grid-template-columns: 1fr 200px 200px auto;
                    align-items: center;
                    padding-left: 18px;
                    height: 40px;
                    box-sizing: border-box;
                    border: solid 1px #ffffff;
                }
                .bar:focus-within {
                    border-color: var(--dark-blue-3);
                }
                ::slotted([slot="query"]) {
                    border: none;
                    padding: 0 10px;
                    height: calc(100% - 2px);
                    font-size: inherit;
                    color: var(--dark-gray-6);
                }
                ::slotted([slot="query"])::placeholder {
                    color: var(--light-gray-4);
                }
                ::slotted([slot="query"]:focus) {
                    outline: none;
                }
                ::slotted([slot="age"]),
                ::slotted([slot="language"]) {
                    border-left: solid 2px var(--light-gray-2);
                }
                ::slotted([slot="button"]) {
                    /* cover .bar border */
                    margin: -1px -2px 0 0;
                    height: 40px;
                    font-size: 16px;
                    font-weight: 600;
                }

                .advanced {
                    position: absolute;
                    /* add 32px for margin */
                    left: calc(100% + 32px);
                    text-align: center;
                }

                /* mobile */
                @media (max-width: 1000px) {
                    .bar {
                        border-radius: 12px;
                        padding: 10px;
                        row-gap: 22px;
                        grid-template-columns: 50% 50%;
                        grid-template-rows: repeat(3, auto);
                        height: unset;
                    }

                    ::slotted([slot="query"]) {
                        grid-column: 1 / span 2;
                        border-bottom: solid 1px var(--light-gray-2);
                        padding: 10px;
                        font-size: 18px;
                    }

                    ::slotted([slot="age"]) {
                        border-left: none;
                    }

                    ::slotted([slot="age"]),
                    ::slotted([slot="language"]) {
                        font-size: 14px;
                    }

                    ::slotted([slot="button"]) {
                        /* cover .bar border */
                        height: unset;
                        font-size: unset;
                        grid-column: 1 / span 2;
                        justify-self: center;
                    }
                }
            `,
        ];
    }

    @property()
    label: string = "";

    render() {
        return html`
            <div class="bar">
                <slot name="query"></slot>
                <slot name="age"></slot>
                <slot name="language"></slot>
                <slot name="button"></slot>
            </div>
            <div class="advanced">
                <slot name="advanced"></slot>
            </div>
        `;
    }
}
