import React from 'react';

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'pink' | 'green';
  children: React.ReactNode;
}

export const Button: React.FC<ButtonProps> = ({ variant = 'primary', children, className = '', ...props }) => {
  const baseStyle = "font-mono text-xs uppercase tracking-wider px-3 py-2 transition-all duration-200 [border-radius:0px_!important] border-2 shadow-[2px_2px_0px_0px_rgba(0,0,0,0.4)] flex items-center justify-center gap-1.5 cursor-pointer";
  
  let variantStyle = "";
  if (variant === 'primary') {
    variantStyle = "bg-cyber-neonBlue/10 hover:bg-cyber-neonBlue/20 border-cyber-neonBlue/50 hover:border-cyber-neonBlue text-cyber-neonBlue hover:shadow-[2px_2px_0px_0px_var(--cyber-neonBlue)]";
  } else if (variant === 'green') {
    variantStyle = "bg-cyber-neonGreen/10 hover:bg-cyber-neonGreen/20 border-cyber-neonGreen/40 hover:border-cyber-neonGreen text-cyber-neonGreen hover:shadow-[2px_2px_0px_0px_var(--cyber-neonGreen)]";
  } else if (variant === 'pink') {
    variantStyle = "bg-cyber-neonPink/10 hover:bg-cyber-neonPink/20 border-cyber-neonPink/40 hover:border-cyber-neonPink text-cyber-neonPink hover:shadow-[2px_2px_0px_0px_var(--cyber-neonPink)]";
  } else {
    variantStyle = "bg-cyber-bg border-cyber-border text-cyber-text hover:border-cyber-text hover:shadow-[2px_2px_0px_0px_rgba(255,255,255,0.2)]";
  }

  return (
    <button className={`${baseStyle} ${variantStyle} ${className}`} {...props}>
      {children}
    </button>
  );
};
