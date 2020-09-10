console.log("process.cwd() = " + process.cwd());
console.log("__dirname = " + __dirname);

module.exports = {
  purge: [
    './src/**/*.js',
    './src/**/*.ts',
    '../templates/**/*.html',
    //this isn't really valid when running from _core itself
    //but doesn't seem to break anything
    '../../_core/templates/**/*.html',
  ],
  theme: {
    extend: {
      colors: {
        jiblueLight: '#83aef7',
        jiblueMedium: '#6698ed',
        jiblueDark: '#2b54b8',
        jinumberblue: '#c4dbfe',
        jigreen: '#61D592',
        jibuttonBlue:'#5590fc',
        jibackgroundBlue: "#e6f0ff",
        jibluelighter: "#a6c6f8",
        jiborderGrey: "#e5e7ef",
        jibackgroundGrey:'#f8f9fd',
        jigreyinputborder:'#d3d3d3;',
        jierrorred: '#e36486',
        jiyellowbackground: '#fdf5d0',
        jiyellowstar:'#fccd63',
        jigreyicon: '#d6d8de',
        jiorange:'#fc7551',
        jisideblue:'#def4ff',
      },


      fontSize: {
        14:'14px',
        18: '18px',

      },
      fontFamily: {
        poppins: 'Poppins',
      },
      width: {
        30: '30px',
        76: '76px',
        112: '112px',
        117: '117px',
        150: '150px',
        176: '176px',
        190: '190px',
        259: '259px',
        272:'272px',
        288: '288px',
        297: '297px',

      },
      height: {
        216: '216px',
        185: '185px',
        696: '696px',
        20: '20px',

      },
      maxHeight: {
        284: '284px'
      },
      borderRadius: {
        20: '20px'
      },
      inset: {
        40:'40px',
        10:'10px',
        25:'25%',
        '-10': '-10px',
        50:'50px',
        '50p':'50%',
        '70p': '70%',
        20: '20px'
      },
      borderWidth: {
        3: '3px'
      },
      backgroundImage: theme => ({
        'shapes': "url('https://i.ibb.co/g9N7MLy/shapes-1.png')",

        })
    },
  },
  variants: {
    backgroundColor: ['responsive', 'hover', 'focus', 'active', 'group-hover'],
    border: ['responsive', 'hover', 'focus', 'active', 'group-hover'],
    transitionProperty: ['responsive', 'motion-safe', 'motion-reduce'],
  },
  plugins: [],
}
