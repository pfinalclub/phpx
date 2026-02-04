// 平滑滚动
document.addEventListener('DOMContentLoaded', function() {
    // 平滑滚动到锚点
    const navLinks = document.querySelectorAll('a[href^="#"]');
    navLinks.forEach(link => {
        link.addEventListener('click', function(e) {
            e.preventDefault();
            const targetId = this.getAttribute('href');
            const targetElement = document.querySelector(targetId);
            if (targetElement) {
                const offsetTop = targetElement.offsetTop - 80; // 考虑固定导航栏高度
                window.scrollTo({
                    top: offsetTop,
                    behavior: 'smooth'
                });
            }
        });
    });

    // 导航栏滚动效果
    const header = document.querySelector('.header');
    let lastScrollY = window.scrollY;

    window.addEventListener('scroll', function() {
        const currentScrollY = window.scrollY;
        
        // 向下滚动时隐藏导航栏，向上滚动时显示
        if (currentScrollY > lastScrollY && currentScrollY > 100) {
            header.style.transform = 'translateY(-100%)';
        } else {
            header.style.transform = 'translateY(0)';
        }
        
        lastScrollY = currentScrollY;
    });

    // 终端模拟器动画
    const terminalLines = document.querySelectorAll('.terminal-line');
    let delay = 0;
    
    terminalLines.forEach(line => {
        line.style.opacity = '0';
        line.style.transform = 'translateY(20px)';
        
        setTimeout(() => {
            line.style.transition = 'all 0.5s ease';
            line.style.opacity = '1';
            line.style.transform = 'translateY(0)';
        }, delay);
        
        delay += 500;
    });

    // 特性卡片动画
    const observerOptions = {
        threshold: 0.1,
        rootMargin: '0px 0px -50px 0px'
    };

    const observer = new IntersectionObserver(function(entries) {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.style.opacity = '1';
                entry.target.style.transform = 'translateY(0)';
            }
        });
    }, observerOptions);

    // 观察特性卡片
    const featureCards = document.querySelectorAll('.feature-card');
    featureCards.forEach((card, index) => {
        card.style.opacity = '0';
        card.style.transform = 'translateY(30px)';
        card.style.transition = 'all 0.6s ease ' + (index * 0.1) + 's';
        observer.observe(card);
    });

    // 代码块语法高亮
    const codeBlocks = document.querySelectorAll('code');
    codeBlocks.forEach(block => {
        // 简单的语法高亮
        const code = block.textContent;
        const highlighted = code
            .replace(/(#.*)/g, '<span class="comment">$1</span>') // 注释
            .replace(/(phpx\s+[^\s]+)/g, '<span class="command">$1</span>') // 命令
            .replace(/(--?[\w-]+)/g, '<span class="option">$1</span>') // 选项
            .replace(/(\$)/g, '<span class="prompt">$1</span>') // 提示符
            .replace(/('[^']*'|"[^"]*")/g, '<span class="string">$1</span>'); // 字符串
        
        block.innerHTML = highlighted;
    });

    // 复制代码功能
    const copyButtons = document.createElement('div');
    copyButtons.className = 'copy-buttons';
    
    document.querySelectorAll('.code-block').forEach(block => {
        const button = document.createElement('button');
        button.className = 'copy-btn';
        button.innerHTML = '📋';
        button.title = '复制代码';
        
        button.addEventListener('click', function() {
            const code = block.querySelector('code').textContent;
            navigator.clipboard.writeText(code).then(() => {
                button.innerHTML = '✅';
                setTimeout(() => {
                    button.innerHTML = '📋';
                }, 2000);
            });
        });
        
        block.style.position = 'relative';
        button.style.position = 'absolute';
        button.style.top = '10px';
        button.style.right = '10px';
        button.style.background = 'rgba(255, 255, 255, 0.1)';
        button.style.border = 'none';
        button.style.color = 'white';
        button.style.padding = '5px 10px';
        button.style.borderRadius = '3px';
        button.style.cursor = 'pointer';
        button.style.fontSize = '14px';
        
        block.appendChild(button);
    });

    // 平台链接动态更新
    const updatePlatformLinks = function() {
        const platformLinks = document.querySelectorAll('.platform-link');
        
        // 这里可以动态获取最新的发布版本链接
        // 暂时使用占位符
        platformLinks.forEach(link => {
            const platform = link.textContent.toLowerCase();
            link.href = `https://github.com/pfinalcub/phpx/releases/latest/download/phpx-${platform}`;
        });
    };
    
    updatePlatformLinks();

    // 深色模式支持
    const darkModeToggle = document.createElement('button');
    darkModeToggle.className = 'dark-mode-toggle';
    darkModeToggle.innerHTML = '🌙';
    darkModeToggle.title = '切换深色模式';
    
    darkModeToggle.style.position = 'fixed';
    darkModeToggle.style.bottom = '20px';
    darkModeToggle.style.right = '20px';
    darkModeToggle.style.background = '#2563eb';
    darkModeToggle.style.border = 'none';
    darkModeToggle.style.color = 'white';
    darkModeToggle.style.padding = '10px';
    darkModeToggle.style.borderRadius = '50%';
    darkModeToggle.style.cursor = 'pointer';
    darkModeToggle.style.fontSize = '18px';
    darkModeToggle.style.zIndex = '1000';
    
    darkModeToggle.addEventListener('click', function() {
        document.body.classList.toggle('dark-mode');
        darkModeToggle.innerHTML = document.body.classList.contains('dark-mode') ? '☀️' : '🌙';
        
        // 保存用户偏好
        localStorage.setItem('darkMode', document.body.classList.contains('dark-mode'));
    });
    
    // 检查用户偏好
    if (localStorage.getItem('darkMode') === 'true') {
        document.body.classList.add('dark-mode');
        darkModeToggle.innerHTML = '☀️';
    }
    
    document.body.appendChild(darkModeToggle);

    // 添加深色模式样式
    const darkModeStyles = `
        .dark-mode {
            background: #0f172a;
            color: #e2e8f0;
        }
        
        .dark-mode .header {
            background: #1e293b;
            border-bottom-color: #334155;
        }
        
        .dark-mode .feature-card,
        .dark-mode .method-card,
        .dark-mode .example,
        .dark-mode .doc-link {
            background: #1e293b;
            border-color: #334155;
            color: #e2e8f0;
        }
        
        .dark-mode .feature-card h3,
        .dark-mode .method-card h3,
        .dark-mode .example h3,
        .dark-mode .doc-title {
            color: #f1f5f9;
        }
        
        .dark-mode .features,
        .dark-mode .usage {
            background: #0f172a;
        }
        
        .dark-mode .footer {
            background: #0f172a;
        }
    `;
    
    const styleSheet = document.createElement('style');
    styleSheet.textContent = darkModeStyles;
    document.head.appendChild(styleSheet);
});

// 页面加载动画
window.addEventListener('load', function() {
    document.body.style.opacity = '0';
    document.body.style.transition = 'opacity 0.3s ease';
    
    setTimeout(() => {
        document.body.style.opacity = '1';
    }, 100);
});