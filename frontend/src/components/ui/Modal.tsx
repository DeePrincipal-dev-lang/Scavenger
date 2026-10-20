/**
 * Modal — shared accessible modal wrapper (issue #875)
 *
 * Built on top of the existing Radix-backed Dialog primitives so all
 * instances get:
 *   - Automatic focus trap (Radix handles this)
 *   - Esc key closes the modal
 *   - Focus restored to the trigger element on close
 *   - role="dialog" + aria-modal="true" (Radix adds these)
 *   - Visible close button with aria-label
 *
 * Usage:
 *   <Modal open={open} onClose={handleClose} title="My Modal" description="...">
 *     {content}
 *   </Modal>
 */
import * as React from 'react'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from '@/components/ui/Dialog'

export interface ModalProps {
  open: boolean
  onClose: () => void
  /** Visible heading — also used as the accessible name for the dialog */
  title: React.ReactNode
  /** Optional description shown below the title */
  description?: React.ReactNode
  children: React.ReactNode
  className?: string
  /**
   * Ref of the element that triggered the modal.
   * When provided, focus is returned to this element on close.
   * Radix handles this automatically when DialogTrigger is used;
   * this prop is for imperative open patterns.
   */
  triggerRef?: React.RefObject<HTMLElement>
}

export function Modal({
  open,
  onClose,
  title,
  description,
  children,
  className,
  triggerRef,
}: ModalProps) {
  // When the dialog closes, return focus to the trigger element if supplied.
  // Radix already does this when DialogTrigger is used; this handles the
  // imperative open pattern (e.g. onClick → setState).
  const handleOpenChange = React.useCallback(
    (isOpen: boolean) => {
      if (!isOpen) {
        onClose()
        // Defer so Radix finishes unmounting before we move focus
        if (triggerRef?.current) {
          setTimeout(() => triggerRef.current?.focus(), 0)
        }
      }
    },
    [onClose, triggerRef]
  )

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className={className}>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          {description && <DialogDescription>{description}</DialogDescription>}
        </DialogHeader>
        {children}
      </DialogContent>
    </Dialog>
  )
}
