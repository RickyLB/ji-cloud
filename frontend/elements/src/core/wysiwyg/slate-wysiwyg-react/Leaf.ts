import React from "react";
import { ReactElement } from "react";
import { RenderLeafProps } from "slate-react";
import { getLeafProps, getLeafStyles } from "../styles";

export function Leaf(props: RenderLeafProps): ReactElement {
    const type = props.leaf.element;

    return React.createElement("span", {
        type,
        style: getLeafStyles(props.leaf),
        ...getLeafProps(props.leaf),
        ...props.attributes,
        children: props.children,
    });
}
