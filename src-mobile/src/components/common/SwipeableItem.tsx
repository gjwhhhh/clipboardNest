import { useState, useRef } from "react";
import { Trash2, Star } from "lucide-react";

interface Props {
  children: React.ReactNode;
  onDelete?: () => void;
  onFavorite?: () => void;
}

export function SwipeableItem({ children, onDelete, onFavorite }: Props) {
  const [translateX, setTranslateX] = useState(0);
  const [startX, setStartX] = useState(0);
  const itemRef = useRef<HTMLDivElement>(null);

  const handleTouchStart = (e: React.TouchEvent) => {
    setStartX(e.touches[0].clientX);
  };

  const handleTouchMove = (e: React.TouchEvent) => {
    const currentX = e.touches[0].clientX;
    const diff = currentX - startX;

    // 限制滑动范围
    if (diff < -100) setTranslateX(-100);
    else if (diff > 100) setTranslateX(100);
    else setTranslateX(diff);
  };

  const handleTouchEnd = () => {
    // 根据滑动距离决定操作
    if (translateX < -50 && onDelete) {
      onDelete();
    } else if (translateX > 50 && onFavorite) {
      onFavorite();
    }
    setTranslateX(0);
  };

  return (
    <div className="relative overflow-hidden">
      {/* 背景操作按钮 */}
      <div className="absolute inset-0 flex">
        <div className="flex-1 bg-yellow-500 flex items-center justify-center">
          <Star className="w-6 h-6 text-white" />
        </div>
        <div className="flex-1 bg-red-500 flex items-center justify-center">
          <Trash2 className="w-6 h-6 text-white" />
        </div>
      </div>

      {/* 内容 */}
      <div
        ref={itemRef}
        className="relative bg-white dark:bg-gray-900 transition-transform"
        style={{ transform: `translateX(${translateX}px)` }}
        onTouchStart={handleTouchStart}
        onTouchMove={handleTouchMove}
        onTouchEnd={handleTouchEnd}
      >
        {children}
      </div>
    </div>
  );
}
