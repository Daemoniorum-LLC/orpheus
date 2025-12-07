import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
  Button,
} from '@persona-framework/ui';
import { CheckCircle2, XCircle, AlertTriangle, Info } from 'lucide-react';

export type MessageType = 'success' | 'error' | 'warning' | 'info';

interface MessageDialogProps {
  open: boolean;
  onClose: () => void;
  title: string;
  message: string;
  type?: MessageType;
}

export function MessageDialog({ open, onClose, title, message, type = 'info' }: MessageDialogProps) {
  const getIcon = () => {
    switch (type) {
      case 'success':
        return <CheckCircle2 className="h-5 w-5 text-green-500" />;
      case 'error':
        return <XCircle className="h-5 w-5 text-red-500" />;
      case 'warning':
        return <AlertTriangle className="h-5 w-5 text-yellow-500" />;
      case 'info':
      default:
        return <Info className="h-5 w-5 text-blue-500" />;
    }
  };

  return (
    <Dialog open={open} onOpenChange={(isOpen) => !isOpen && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            {getIcon()}
            <span>{title}</span>
          </DialogTitle>
          <DialogDescription className="whitespace-pre-wrap leading-relaxed">
            {message}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="default" onClick={onClose}>
            OK
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
