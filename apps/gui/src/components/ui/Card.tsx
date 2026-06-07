import React from 'react';

interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode;
}

export const Card: React.FC<CardProps> = ({ children, className = '', ...props }) => {
  return (
    <div
      className={`border-2 border-cyber-border bg-cyber-panel/60 backdrop-blur-sm p-4 transition-all duration-200 select-none [border-radius:0px_!important] shadow-[4px_4px_0px_0px_rgba(0,0,0,0.4)] hover:shadow-[4px_4px_0px_0px_var(--cyber-neonBlue)] ${className}`}
      {...props}
    >
      {children}
    </div>
  );
};
