import type { PropsWithChildren, ReactNode } from 'react';

export function Card({ title, actions, children, className = '' }: PropsWithChildren<{ title: string; actions?: ReactNode; className?: string }>) {
  return (
    <section className={`card ${className}`.trim()}>
      <header className="card-header">
        <div>
          <p className="eyebrow">Feature</p>
          <h3>{title}</h3>
        </div>
        {actions}
      </header>
      {children}
    </section>
  );
}

export function Pill({ children, tone = 'neutral' }: PropsWithChildren<{ tone?: 'neutral' | 'accent' | 'warning' | 'danger' | 'success' }>) {
  return <span className={`pill pill-${tone}`}>{children}</span>;
}

export function Stat({ label, value, hint }: { label: string; value: ReactNode; hint?: string }) {
  return (
    <div className="stat">
      <span>{label}</span>
      <strong>{value}</strong>
      {hint ? <small>{hint}</small> : null}
    </div>
  );
}
