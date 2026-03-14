import { useEffect, useRef, useState } from "react";
import { useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { AlertCircle, Loader2, Video } from "lucide-react";
import { useAuthStore } from "@/stores/auth";
import apiClient from "@/lib/api-client";

interface MediaInfo {
  proxy_url: string;
  thumbnail_url: string | null;
  duration: number | null;
  width: number | null;
  height: number | null;
}

export function VideoPlayer({ token }: { token: string }) {
  const { t } = useTranslation();
  const { botId } = useParams<{ botId: string }>();
  const [mediaInfo, setMediaInfo] = useState<MediaInfo | null>(null);
  const [playing, setPlaying] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(false);
  const [blobUrl, setBlobUrl] = useState<string | null>(null);
  const blobUrlRef = useRef<string | null>(null);

  // Clean up blob URL on unmount
  useEffect(() => {
    return () => {
      if (blobUrlRef.current) {
        URL.revokeObjectURL(blobUrlRef.current);
      }
    };
  }, []);

  // Fetch media info on click (lazy -- no requests until user interacts)
  const handlePlay = () => {
    if (blobUrl) {
      setPlaying(true);
      return;
    }
    if (mediaInfo) {
      // Have media info but no blob yet -- fetch the video
      fetchVideoBlob(mediaInfo);
      return;
    }
    if (!botId || loading) return;

    setLoading(true);
    setError(false);
    apiClient
      .get<MediaInfo>(`/bots/${botId}/media-info/${token}`)
      .then((res) => {
        setMediaInfo(res.data);
        fetchVideoBlob(res.data);
      })
      .catch(() => {
        setMediaInfo(null);
        setPlaying(false);
        setError(true);
        setLoading(false);
      });
  };

  const fetchVideoBlob = (info: MediaInfo) => {
    const { accessToken } = useAuthStore.getState();
    setLoading(true);
    setError(false);

    fetch(info.proxy_url, {
      headers: { Authorization: `Bearer ${accessToken}` },
    })
      .then((response) => {
        if (!response.ok) throw new Error("Fetch failed");
        return response.blob();
      })
      .then((blob) => {
        // Revoke previous blob URL if any
        if (blobUrlRef.current) {
          URL.revokeObjectURL(blobUrlRef.current);
        }
        const url = URL.createObjectURL(blob);
        blobUrlRef.current = url;
        setBlobUrl(url);
        setPlaying(true);
      })
      .catch(() => {
        setPlaying(false);
        setError(true);
      })
      .finally(() => setLoading(false));
  };

  const formatDuration = (s: number) => {
    const m = Math.floor(s / 60);
    const sec = s % 60;
    return `${m}:${sec.toString().padStart(2, "0")}`;
  };

  if (playing && blobUrl) {
    return (
      <video
        src={blobUrl}
        controls
        autoPlay
        aria-label={t("common.playVideo")}
        className="max-h-64 max-w-xs rounded-xl"
      />
    );
  }

  // Thumbnail preview with play button
  return (
    <button
      onClick={handlePlay}
      disabled={loading}
      aria-label={t("common.playVideo")}
      className="block relative max-w-xs rounded-xl overflow-hidden group text-left"
    >
      {mediaInfo?.thumbnail_url ? (
        <img
          src={mediaInfo.thumbnail_url}
          alt=""
          referrerPolicy="no-referrer"
          className="max-h-48 object-cover"
        />
      ) : (
        <div className="w-48 h-32 bg-muted/50 flex items-center justify-center">
          <Video className="size-6 text-muted-foreground" />
        </div>
      )}
      {/* Play overlay */}
      <div className="absolute inset-0 flex items-center justify-center bg-black/20 group-hover:bg-black/30 transition-colors">
        {loading ? (
          <Loader2 className="size-8 text-white animate-spin" />
        ) : error ? (
          <div className="flex flex-col items-center gap-1">
            <AlertCircle className="size-8 text-red-400" />
            <span className="text-[10px] text-white bg-black/50 rounded px-1.5 py-0.5">
              {t("errors.somethingWentWrong")}
            </span>
          </div>
        ) : (
          <div className="size-10 rounded-full bg-white/90 flex items-center justify-center shadow-lg">
            <div className="w-0 h-0 border-t-[6px] border-t-transparent border-b-[6px] border-b-transparent border-l-[10px] border-l-black/80 ml-0.5" />
          </div>
        )}
      </div>
      {/* Duration badge */}
      {mediaInfo?.duration && (
        <span className="absolute bottom-1.5 right-1.5 text-[10px] bg-black/70 text-white px-1.5 py-0.5 rounded">
          {formatDuration(mediaInfo.duration)}
        </span>
      )}
    </button>
  );
}
