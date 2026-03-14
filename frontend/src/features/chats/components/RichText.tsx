import type { MarkupElement } from "@/features/chats/lib/payload";

const URL_REGEX = /(https?:\/\/[^\s<>"'()]+[^\s<>"'().,;:!?])/;

export function RichText({
  text,
  markup,
}: {
  text: string;
  markup?: MarkupElement[];
}) {
  if (!markup || markup.length === 0) {
    return <Linkify text={text} />;
  }

  // Sort markup by position, build segments
  const sorted = [...markup].sort((a, b) => a.from - b.from);
  const result: React.ReactNode[] = [];
  let cursor = 0;

  for (let i = 0; i < sorted.length; i++) {
    const m = sorted[i];
    const to = m.from + m.length;

    // Defensive fix for overlapping ranges: clamp start to cursor
    const effectiveFrom = Math.max(m.from, cursor);
    if (effectiveFrom >= to) continue;

    // Text before this markup
    if (effectiveFrom > cursor) {
      result.push(
        <Linkify key={`t${i}`} text={text.slice(cursor, effectiveFrom)} />,
      );
    }

    const content = text.slice(effectiveFrom, to);

    switch (m.type) {
      case "strong":
        result.push(<strong key={i}>{content}</strong>);
        break;
      case "emphasized":
        result.push(<em key={i}>{content}</em>);
        break;
      case "monospaced":
        result.push(
          <code
            key={i}
            className="bg-muted px-1 py-0.5 rounded text-xs font-mono"
          >
            {content}
          </code>,
        );
        break;
      case "strikethrough":
        result.push(
          <s key={i} className="text-muted-foreground">
            {content}
          </s>,
        );
        break;
      case "underline":
        result.push(<u key={i}>{content}</u>);
        break;
      case "link": {
        const href = m.url ?? content;
        const isValidUrl = /^(?:https?|mailto):/.test(href);
        result.push(
          isValidUrl ? (
            <a
              key={i}
              href={href}
              target="_blank"
              rel="noopener noreferrer"
              className="text-primary underline underline-offset-2 hover:text-primary/80"
            >
              {content}
            </a>
          ) : (
            <span key={i}>{content}</span>
          ),
        );
        break;
      }
      case "user_mention":
        result.push(
          <span key={i} className="text-primary font-medium">
            {content}
          </span>,
        );
        break;
      case "heading":
        result.push(
          <span key={i} className="text-base font-semibold">
            {content}
          </span>,
        );
        break;
      case "highlighted":
        result.push(
          <mark
            key={i}
            className="bg-yellow-200/40 dark:bg-yellow-500/20 rounded px-0.5"
          >
            {content}
          </mark>,
        );
        break;
      default:
        result.push(<span key={i}>{content}</span>);
    }

    cursor = to;
  }

  // Remaining text after last markup
  if (cursor < text.length) {
    result.push(<Linkify key="tail" text={text.slice(cursor)} />);
  }

  return <>{result}</>;
}

function Linkify({ text }: { text: string }) {
  const parts = text.split(URL_REGEX);
  return (
    <>
      {parts.map((part, i) =>
        URL_REGEX.test(part) ? (
          <a
            key={i}
            href={part}
            target="_blank"
            rel="noopener noreferrer"
            className="text-primary underline underline-offset-2 hover:text-primary/80 break-all"
          >
            {part}
          </a>
        ) : (
          <span key={i}>{part}</span>
        ),
      )}
    </>
  );
}
