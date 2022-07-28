import { LitElement, html, css, customElement, property } from "lit-element";

export type Kind = "jig" | "resource" | "course";

const STR_JIGS = "JIGs";
const STR_RESOURCES = "Resource Library";
const STR_COURSES = "Courses";

const IMAGE_LOOKUP: {
    [key in Kind]: string;
} = {
    ["jig"]: "jig-section.png",
    ["resource"]: "resources.webp",
    ["course"]: "course-section.svg",
};

@customElement("home-search-results-section")
export class _ extends LitElement {
    static get styles() {
        return [
            css`
                :host {
                    display: grid;
                    row-gap: 48px;
                    padding: 5px 60px;
                    max-width: 1800px;
                    margin: 0 auto;
                }
                :host([kind=resource]) {
                    background-color: var(--green-2);
                }
                .top-line {
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                }
                .left-side {
                    display: flex;
                    position: relative;
                }
                .left-side img-ui {
                    position: absolute;
                    right: 100%;
                    height: 55px;
                }
                h2 {
                    margin: 0;
                    font-size: 40px;
                    font-weight: 800;
                    color: var(--dark-blue-4);
                }
                .results-count {
                    font-size: .6em;
                    font-weight: 500;
                }
                .results {
                    /* display: flex;
                    flex-wrap: wrap; */

                    display: grid;
                    grid-template-columns: repeat(auto-fill, 354px);

                    justify-content: space-between;
                    row-gap: 80px;
                    column-gap: 40px;
                }
                .load-more {
                    display: grid;
                    place-content: center;
                }
                .load-more ::slotted(*) {
                    margin-bottom: 40px;
                }

                /* mobile */
                @media (max-width: 1000px) {
                    :host {
                        padding: 5px;
                    }
                    .top-line {
                        justify-content: center;
                    }
                    .left-side img-ui {
                        position: static;
                    }
                    h2 {
                        font-size: 40px;
                    }
                    .results {
                        justify-content: center;
                    }
                }
            `,
        ];
    }

    @property({ reflect: true })
    kind: Kind = "jig";

    @property({ type: Number })
    resultsCount: number = 0;

    render() {
        return html`
            <div class="top-line">
                <div class="left-side">
                    <img-ui
                        path="entry/home/search-results/${IMAGE_LOOKUP[this.kind]}"
                    ></img-ui>
                    <h2>
                        ${
                            this.kind === "jig" ? STR_JIGS
                                : this.kind === "resource" ? STR_RESOURCES
                                : STR_COURSES
                        }
                        <span class="results-count">
                            (${this.resultsCount})
                        </span
                        >
                    </h2>
                </div>
                <slot name="sort"></slot>
            </div>
            <div class="results">
                <slot name="results"></slot>
            </div>
            <div class="load-more">
                <slot name="load-more"></slot>
            </div>
        `;
    }
}
