// Toast 提示组件
import { useEffect, useState } from "react";
import { CheckCircle, AlertCircle, X } from "lucide-react";

export interface ToastMessage {
  id: string;
  type: "success" | "error";
  message: string;
}

interface Props {
  toast: ToastMessage | null;
  onClose: () => void;
}

export function Toast({ toast, onClose }: Props) {
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    if (toast) {
      setVisible(true);
      const timer = setTimeout(() => {
        setVisible(false);
        setTimeout(onClose, 300);
      }, 2500);
      return () => clearTimeout(timer);
    }
  }, [toast, onClose]);

  if (!toast) return null;

  return (
    <div
      className={`
        fixed top-4 right-4 z-[100] flex items-center gap-2 px-4 py-2.5
        rounded-lg shadow-lg transition-all duration-300
        ${visible ? "opacity-100 translate-y-0" : "opacity-0 -translate-y-2"}
        ${toast.type === "success"
          ? "bg-green-500 text-white"
          : "bg-red-500 text-white"
        }
      `}
    >
      {toast.type === "success" ? (
        <CheckCircle className="w-4 h-4" />
      ) : (
        <AlertCircle className="w-4 h-4" />
      )}
      <span className="text-sm">{toast.message}</span>
      <button
        onClick={() => {
          setVisible(false);
          setTimeout(onClose, 300);
        }}
        className="ml-2 p-0.5 hover:bg-white/20 rounded"
      >
        <X className="w-3 h-3" />
      </button>
    </div>
  );
}
