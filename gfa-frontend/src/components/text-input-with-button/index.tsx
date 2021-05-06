import { FunctionalComponent, h } from 'preact';
import { useState } from 'preact/hooks';
import * as style from './style.css';

interface TextInputWithButtonProps {
  label: string;
  completed: boolean;
  loading: boolean;
  error: string;
  onButtonClick: (text: string) => void;
  placeholder: string;
  type?: string;
}

const TextInputWithButton: FunctionalComponent<TextInputWithButtonProps> = ({
  label,
  completed,
  loading,
  error,
  onButtonClick,
  placeholder,
  type
}) => {
  const [text, setText] = useState<string>('');
  return (
    <div>
      <form>
        <input
          type={type || 'text'}
          placeholder={placeholder}
          onInput={(event): void => {
            setText((event.target as HTMLInputElement).value);
          }}
        />
        <button
          onClick={(event): void => {
            event.preventDefault();
            onButtonClick(text);
          }}
        >
          {!completed && !loading && <div>{label}</div>}
          {completed && !loading && <div>✓</div>}
          {loading && <div className={style.loader} />}
        </button>
      </form>
      {error && <div className={style.error}>{error}</div>}
    </div>
  );
};

export default TextInputWithButton;
