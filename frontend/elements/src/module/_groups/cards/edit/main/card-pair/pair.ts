import { LitElement, html, css, customElement, property } from "lit-element";
import "@elements/core/hebrew-buttons/hebrew-buttons";

@customElement("main-card-pair")
export class _ extends LitElement {
    static get styles() {
        return [
            css`
                section,
                .index {
                    width: 350px;
                }

                section {
                    border-radius: 24rem;
                    height: 236px;
                }

                .close {
                    display: none;
                    position: relative;
                    top: 0;
                    left: 0;
                }
                slot[name="close"]::slotted(*) {
                    position: absolute;
                    top: -16px;
                    left: 333px;
                    display: inline-block;
                    width: 32px;
                    height: 32px;
                }

                .cards {
                    display: grid;
                    padding: 27px;
                    grid-template-columns: 1fr 24px 1fr;
                    grid-template-rows: 24px 1fr 24px;
                }
                .left {
                    grid-row: 1 / span 2;
                    grid-column: 1 / span 2;
                }
                .right {
                    grid-row: 2 / span 2;
                    grid-column: 2 / span 2;
                }
                ::slotted(hebrew-buttons) {
                    align-self: center;
                    justify-self: end;
                    grid-row: 1;
                    grid-column: 1 / 4;
                    margin-bottom: 70px;
                    background-color: #fff;
                    padding: 10px;
                    box-shadow: 0 3px 16px 0 rgba(0, 0, 0, 0.2);
                    border-radius: 8px;
                    z-index: 1;
                }
                .index {
                    font-size: 14px;
                    text-align: center;
                }
            `,
        ];
    }

    @property({ type: Number })
    index: number = 0;

    render() {
        const { index } = this;

        return html`
            <section>
                <div class="close"><slot name="close"></slot></div>
                <div class="cards">
                    <div class="right"><slot name="right"></slot></div>
                    <div class="left"><slot name="left"></slot></div>
                    <slot name="hebrew-buttons"></slot>
                </div>
            </section>
            <div class="index">${index + 1}</div>
            <slot name="error"></slot>
        `;
    }
}
