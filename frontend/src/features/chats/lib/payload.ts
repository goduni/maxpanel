import type { BotEvent } from "@/lib/api-types";

export interface MarkupElement {
  type: string;
  from: number;
  length: number;
  url?: string;
  user_link?: string;
  user_id?: number;
}

export function getMarkup(body: Record<string, unknown>): MarkupElement[] {
  const raw = body.markup;
  if (!Array.isArray(raw)) return [];
  return raw.filter((m): m is MarkupElement =>
    typeof m === "object" && m !== null &&
    typeof (m as Record<string, unknown>).from === "number" &&
    typeof (m as Record<string, unknown>).length === "number" &&
    typeof (m as Record<string, unknown>).type === "string"
  );
}

export function getSenderDirect(sender: unknown): { userId: number; name: string } | null {
  if (typeof sender !== "object" || sender === null) return null;
  const s = sender as Record<string, unknown>;
  const userId = typeof s.user_id === "number" ? s.user_id : null;
  if (userId === null) return null;
  const firstName = typeof s.first_name === "string" ? s.first_name : null;
  const lastName = typeof s.last_name === "string" ? s.last_name : null;
  const legacyName = typeof s.name === "string" ? s.name : null;
  const name = [firstName, lastName].filter(Boolean).join(" ") || legacyName || "";
  return { userId, name };
}

export function isOutboundEvent(event: BotEvent): boolean {
  return event.direction === "outbound";
}

export function getPayload(event: BotEvent): Record<string, unknown> | null {
  if (typeof event.raw_payload === "object" && event.raw_payload !== null) {
    return event.raw_payload as Record<string, unknown>;
  }
  return null;
}

export function getMessage(
  payload: Record<string, unknown>,
  event?: BotEvent,
): Record<string, unknown> | null {
  // For outbound events, try multiple payload formats:
  // 1. Direct message (history sync format: { message: {...} })
  // 2. Gateway format (response_body.message)
  // 3. Synthetic from request_body
  if (event && isOutboundEvent(event)) {
    // History sync / proxy format — same as inbound
    if (typeof payload.message === "object" && payload.message !== null) {
      return payload.message as Record<string, unknown>;
    }
    // Gateway format — response_body.message
    const responseBody = payload.response_body;
    if (typeof responseBody === "object" && responseBody !== null) {
      const rb = responseBody as Record<string, unknown>;
      if (typeof rb.message === "object" && rb.message !== null) {
        return rb.message as Record<string, unknown>;
      }
    }
    // Construct a synthetic message from request_body for outbound
    const requestBody = payload.request_body;
    if (typeof requestBody === "object" && requestBody !== null) {
      return {
        body: requestBody,
        sender: null,
      } as Record<string, unknown>;
    }
    return null;
  }

  if (typeof payload.message === "object" && payload.message !== null) {
    return payload.message as Record<string, unknown>;
  }
  return null;
}

export function getBody(
  message: Record<string, unknown>,
): Record<string, unknown> | null {
  if (typeof message.body === "object" && message.body !== null) {
    return message.body as Record<string, unknown>;
  }
  return null;
}

export function getSender(
  message: Record<string, unknown>,
  event?: BotEvent,
): { name: string; userId: number; isBot: boolean } | null {
  // For outbound events, the sender is the bot
  if (event && isOutboundEvent(event)) {
    return {
      name: "Bot",
      userId: 0,
      isBot: true,
    };
  }

  const sender = message.sender;
  if (typeof sender !== "object" || sender === null) return null;
  const s = sender as Record<string, unknown>;
  const firstName = typeof s.first_name === "string" ? s.first_name : null;
  const lastName = typeof s.last_name === "string" ? s.last_name : null;
  // Fallback to deprecated "name" field
  const legacyName = typeof s.name === "string" ? s.name : null;
  const userId = typeof s.user_id === "number" ? s.user_id : 0;
  const isBot = s.is_bot === true;
  const displayName = [firstName, lastName].filter(Boolean).join(" ") || legacyName;
  return {
    name: displayName || `User ${userId}`,
    userId,
    isBot,
  };
}

export function getMid(message: Record<string, unknown>): string | null {
  const body = getBody(message);
  const mid = body ? body.mid : message.mid;
  return typeof mid === "string" ? mid : null;
}

export function getAttachments(
  body: Record<string, unknown>,
): Array<Record<string, unknown>> {
  if (Array.isArray(body.attachments)) {
    return body.attachments as Array<Record<string, unknown>>;
  }
  return [];
}
