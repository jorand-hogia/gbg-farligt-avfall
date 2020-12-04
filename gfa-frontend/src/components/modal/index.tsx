import * as style from './style.css';
import { h, FunctionalComponent } from 'preact';
import type { ComponentChildren } from 'preact';

interface ModalProps {
    isOpen: boolean;
    children: ComponentChildren;
    onClickBackdrop: () => void;
}

export const Modal: FunctionalComponent<ModalProps> = (props: ModalProps) => {
    if (!props.isOpen) {
        return null;
    }
    return (
        <div>
            <div className={style.modal}>{props.children}</div>
            <div className={style.backdrop} onClick={e => {
                e.preventDefault();
                props.onClickBackdrop();
            }}/>
        </div>
    );
}
